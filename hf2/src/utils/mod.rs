use super::{
    checksum_pages, reset_into_app, start_flash, write_flash_page, BinInfoMode, BinInfoResponse,
    Error,
};
use crc_any::CRCu16;
use goblin::elf::program_header::*;
use hidapi::HidDevice;
use std::path::PathBuf;
use std::{fs::File, io::Read};

#[derive(Debug)]
pub enum UtilError {
    File,
    InvalidBinary,
    Elf,
    Internal,
    Communication,
    ContentsDifferent,
}

impl From<Error> for UtilError {
    fn from(err: Error) -> UtilError {
        match err {
            Error::Parse | Error::Transmission => UtilError::Communication,
            _ => UtilError::Internal,
        }
    }
}

/// Returns a contiguous bin with 0s between non-contiguous sections and starting address from an elf.
pub fn elf_to_bin(path: PathBuf) -> Result<(Vec<u8>, u32), UtilError> {
    let mut file = File::open(path).map_err(|_| UtilError::File)?;
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).map_err(|_| UtilError::File)?;

    let binary = goblin::elf::Elf::parse(&buffer.as_slice()).map_err(|_| UtilError::Elf)?;

    let mut start_address: u64 = 0;
    let mut last_address: u64 = 0;

    let mut data = vec![];
    for (i, ph) in binary
        .program_headers
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD && ph.p_filesz > 0)
        .enumerate()
    {
        data.extend_from_slice(&buffer[ph.p_offset as usize..][..ph.p_filesz as usize]);

        if i == 0 {
            start_address = ph.p_paddr;
        }
        //if any of the rest of the sections are non contiguous, fill zeros
        else {
            for _ in 0..(ph.p_paddr - last_address) {
                data.push(0x0);
            }
        }

        last_address = start_address + ph.p_filesz;
    }

    Ok((data, start_address as u32))
}

/// Flash, Verify and restart into app.
pub fn flash_bin(
    binary: &[u8],
    address: u32,
    bininfo: &BinInfoResponse,
    d: &HidDevice,
) -> Result<(), UtilError> {
    if binary.is_empty() {
        return Err(UtilError::InvalidBinary);
    }

    let mut binary = binary.to_owned();

    // pad zeros to page size
    // add divisor-1 to dividend to round up
    let padded_num_pages =
        (binary.len() as u32 + (bininfo.flash_page_size - 1)) / bininfo.flash_page_size;

    let padded_size = padded_num_pages * bininfo.flash_page_size;
    log::debug!(
        "binary is {} bytes, padding to {} bytes",
        binary.len(),
        padded_size
    );
    for _i in 0..(padded_size as usize - binary.len()) {
        binary.push(0x0);
    }

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = start_flash(&d).map_err(UtilError::from)?;
    }
    flash(&binary, address, &bininfo, &d)?;

    match verify(&binary, address, &bininfo, &d) {
        Ok(false) => return Err(UtilError::ContentsDifferent),
        Err(e) => return Err(UtilError::from(e)),
        Ok(true) => (),
    };

    reset_into_app(&d).map_err(UtilError::from)
}

/// Flashes binary writing a single page at a time.
fn flash(
    binary: &[u8],
    address: u32,
    bininfo: &BinInfoResponse,
    d: &HidDevice,
) -> Result<(), UtilError> {
    for (page_index, page) in binary.chunks(bininfo.flash_page_size as usize).enumerate() {
        let target_address = address + bininfo.flash_page_size * page_index as u32;

        let _ = write_flash_page(&d, target_address, page.to_vec()).map_err(UtilError::from)?;
    }
    Ok(())
}

pub fn verify_bin(
    binary: &[u8],
    address: u32,
    bininfo: &BinInfoResponse,
    d: &HidDevice,
) -> Result<(), UtilError> {
    let mut binary = binary.to_owned();

    // pad zeros to page size
    // add divisor-1 to dividend to round up
    let padded_num_pages =
        (binary.len() as u32 + (bininfo.flash_page_size - 1)) / bininfo.flash_page_size;

    let padded_size = padded_num_pages * bininfo.flash_page_size;

    for _i in 0..(padded_size as usize - binary.len()) {
        binary.push(0x0);
    }

    match verify(&binary, address, &bininfo, &d) {
        Ok(false) => Err(UtilError::ContentsDifferent),
        Err(e) => Err(e),
        Ok(true) => Ok(()),
    }
}

/// Verifys checksum of binary.
fn verify(
    binary: &[u8],
    address: u32,
    bininfo: &BinInfoResponse,
    d: &HidDevice,
) -> Result<bool, UtilError> {
    // get checksums of existing pages

    let top_address = address + binary.len() as u32;

    let max_pages = bininfo.max_message_size / 2 - 2;
    let steps = max_pages * bininfo.flash_page_size;
    let mut device_checksums = vec![];

    for target_address in (address..top_address).step_by(steps as usize) {
        // add divisor-1 to dividend to round up
        let pages_left = (top_address - target_address + (bininfo.flash_page_size - 1))
            / bininfo.flash_page_size;

        let num_pages = if pages_left < max_pages {
            pages_left
        } else {
            max_pages
        };

        let chk = checksum_pages(&d, target_address, num_pages).map_err(UtilError::from)?;
        device_checksums.extend_from_slice(&chk.checksums);
    }

    let mut binary_checksums = vec![];

    //collect and sums so we can view all mismatches, not just first
    for page in binary.chunks(bininfo.flash_page_size as usize) {
        let mut xmodem = CRCu16::crc16xmodem();
        xmodem.digest(&page);

        binary_checksums.push(xmodem.get_crc());
    }

    Ok(binary_checksums.eq(&device_checksums))
}

pub fn vendor_map() -> std::collections::HashMap<u16, Vec<u16>> {
    maplit::hashmap! {
        0x1D50 => vec![0x6110, 0x6112],
        0x239A => vec![0x0035, 0x002D, 0x0015, 0x001B, 0xB000, 0x0024, 0x000F, 0x0013, 0x0021, 0x0022, 0x0031, 0x002B, 0x0037, 0x0035, 0x002F, 0x002B, 0x0033, 0x0034, 0x003D, 0x0018, 0x001C, 0x001E, 0x0027, 0x0022],
        0x04D8 => vec![0xEDB3, 0xEDBE, 0xEF66],
        0x2341 => vec![0x024E, 0x8053, 0x024D],
        0x16D0 => vec![0x0CDA],
        0x03EB => vec![0x2402],
        0x2886 => vec![0x000D, 0x002F],
        0x1B4F => vec![0x0D23, 0x0D22],
        0x1209 => vec![0x4D44, 0x2017],
    }
}
