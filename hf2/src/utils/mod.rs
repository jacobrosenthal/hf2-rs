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

    let binary = goblin::elf::Elf::parse(buffer.as_slice()).map_err(|_| UtilError::Elf)?;

    let mut start_address: u64 = 0;
    let mut last_address: u64 = 0;

    let mut data = vec![];
    for (i, ph) in binary
        .program_headers
        .iter()
        .filter(|ph| {
            ph.p_type == PT_LOAD
                && ph.p_filesz > 0
                && ph.p_offset >= binary.header.e_ehsize as u64
                && ph.is_read()
        })
        .enumerate()
    {
        // first time through grab the starting physical address
        if i == 0 {
            start_address = ph.p_paddr;
        }
        // on subsequent passes, if there's a gap between this section and the
        // previous one, fill it with zeros
        else {
            let difference = (ph.p_paddr - last_address) as usize;
            data.resize(data.len() + difference, 0x0);
        }

        data.extend_from_slice(&buffer[ph.p_offset as usize..][..ph.p_filesz as usize]);

        last_address = ph.p_paddr + ph.p_filesz;
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
        let _ = start_flash(d).map_err(UtilError::from)?;
    }
    flash(&binary, address, bininfo, d)?;

    match verify(&binary, address, bininfo, d) {
        Ok(false) => return Err(UtilError::ContentsDifferent),
        Err(e) => return Err(e),
        Ok(true) => (),
    };

    reset_into_app(d).map_err(UtilError::from)
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

        let _ = write_flash_page(d, target_address, page.to_vec()).map_err(UtilError::from)?;
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

    match verify(&binary, address, bininfo, d) {
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

        let chk = checksum_pages(d, target_address, num_pages).map_err(UtilError::from)?;
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
        0x239A => vec![0x007F, 0x00B3, 0x003F, 0x0051, 0x0093, 0x0087, 0x003B, 0x0071, 0x0045, 0x0063, 0x0029, 0x0079, 0x0061, 0xE005, 0x0095, 0x004D, 0x006B, 0x0057, 0x00B5, 0x007D, 0x00B9, 0x0065, 0x0047, 0x0049, 0x00AF, 0x00CD, 0x00BF, 0x00C3, 0x00CB, 0x00AB, 0x00C5, 0x00A5, 0x00A7, 0x00C7, 0x002D, 0x0015, 0x001B, 0xB000, 0x0024, 0x000F, 0x0013, 0x0021, 0x0031, 0x0037, 0x0035, 0x002F, 0x002B, 0x0033, 0x0034, 0x003D, 0x0018, 0x001C, 0x001E, 0x0027, 0x0022, 0x00EF],
        0x04D8 => vec![0xEC44, 0xEC64, 0xEC63, 0xEDB3, 0xEDBE, 0xEF66],
        0x2341 => vec![0x0057, 0x024E, 0x8053, 0x024D],
        0x16D0 => vec![0x0CDA],
        0x03EB => vec![0x2402],
        0x2886 => vec![0x002D, 0x000D, 0x002F],
        0x1B4F => vec![0x0D23, 0x0D22, 0x0016],
        0x1209 => vec![0x805A, 0x7102, 0x4D44, 0x2017],
        0x3171 => vec![0x0100],
        0x1915 => vec![0x521F],
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn elf_rustc_1_44_0() {
        let (_, start_addr) = super::elf_to_bin(
            [
                env!("CARGO_MANIFEST_DIR"),
                "src/utils/testdata/blinky_1.44.0",
            ]
            .iter()
            .collect(),
        )
        .unwrap();
        assert_eq!(start_addr, 0x4000);
    }

    #[test]
    fn elf_rustc_1_47_0() {
        let (_, start_addr) = super::elf_to_bin(
            [
                env!("CARGO_MANIFEST_DIR"),
                "src/utils/testdata/blinky_1.47.0",
            ]
            .iter()
            .collect(),
        )
        .unwrap();
        assert_eq!(start_addr, 0x4000);
    }

    #[test]
    fn elf_sections() {
        let (data, start_addr) = super::elf_to_bin(
            [env!("CARGO_MANIFEST_DIR"), "src/utils/testdata/sections"]
                .iter()
                .collect(),
        )
        .unwrap();
        println!("{:?}", data);
        assert_eq!(start_addr, 0);
        assert_eq!(data[0], 1);
        assert_eq!(data[12], 2);
        assert_eq!(data[20], 3);
    }
}
