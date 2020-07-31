use super::{
    checksum_pages, reset_into_app, start_flash, write_flash_page, BinInfoMode, BinInfoResponse,
};
use crc_any::CRCu16;
use goblin::elf::program_header::*;
use hidapi::HidDevice;
use std::path::PathBuf;
use std::{fs::File, io::Read};

/// Returns a contiguous bin with 0s between non-contiguous sections and starting address from an elf.
pub fn elf_to_bin(path: PathBuf) -> (Vec<u8>, u32) {
    let mut file = File::open(path).unwrap();
    let mut buffer = vec![];
    file.read_to_end(&mut buffer).unwrap();

    let binary = goblin::elf::Elf::parse(&buffer.as_slice()).expect("Couldn't parse elf");

    // we need to fill any noncontigous section space with zeros to send over to uf2 bootloader in one batch (for some reason)
    // todo this is a mess
    let (data, _, start_address) = binary
        .program_headers
        .iter()
        .filter(|ph| ph.p_type == PT_LOAD && ph.p_filesz > 0)
        .fold(
            (vec![], 0x0, 0x0),
            move |(mut data, last_address, start_address), ph| {
                log::debug!("{:?}", ph);

                let current_address = ph.p_filesz + ph.p_paddr;

                //first time through we dont want any of the padding zeros and we want to set the starting address
                if data.is_empty() {
                    data.extend_from_slice(&buffer[ph.p_offset as usize..][..ph.p_filesz as usize]);

                    (data, current_address, ph.p_paddr)
                }
                //other times through pad any space between sections and maintain the starting address
                else {
                    for _ in 0..(current_address - last_address) {
                        data.push(0x0);
                    }

                    data.extend_from_slice(&buffer[ph.p_offset as usize..][..ph.p_filesz as usize]);

                    (data, current_address, start_address)
                }
            },
        );

    (data, start_address as u32)
}

/// Flash, Verify and restart into app.
pub fn flash_bin(binary: &[u8], address: u32, bininfo: &BinInfoResponse, d: &HidDevice) {
    assert!(!binary.is_empty(), "Elf has nothing to flash?");

    let mut binary = binary.to_owned();

    //pad zeros to page size
    let padded_num_pages = (binary.len() as f64 / f64::from(bininfo.flash_page_size)).ceil() as u32;
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
        let _ = start_flash(&d).expect("start_flash failed");
    }
    flash(&binary, address, &bininfo, &d);
    verify(&binary, address, &bininfo, &d);
    let _ = reset_into_app(&d).expect("reset_into_app failed");
}

/// Flashes binary writing a single page at a time.
fn flash(binary: &[u8], address: u32, bininfo: &BinInfoResponse, d: &HidDevice) {
    for (page_index, page) in binary.chunks(bininfo.flash_page_size as usize).enumerate() {
        let target_address = address + bininfo.flash_page_size * page_index as u32;

        let _ =
            write_flash_page(&d, target_address, page.to_vec()).expect("write_flash_page failed");
    }
}

/// Verifys checksum of binary.
pub fn verify(binary: &[u8], address: u32, bininfo: &BinInfoResponse, d: &HidDevice) {
    // get checksums of existing pages
    let top_address = address + binary.len() as u32;
    let max_pages = bininfo.max_message_size / 2 - 2;
    let steps = max_pages * bininfo.flash_page_size;
    let mut device_checksums = vec![];

    for target_address in (address..top_address).step_by(steps as usize) {
        let pages_left = (top_address - target_address) / bininfo.flash_page_size;

        let num_pages = if pages_left < max_pages {
            pages_left
        } else {
            max_pages
        };
        let chk = checksum_pages(&d, target_address, num_pages).expect("checksum_pages failed");
        device_checksums.extend_from_slice(&chk.checksums[..]);
    }

    let mut binary_checksums = vec![];

    //collect and sums so we can view all mismatches, not just first
    for page in binary.chunks(bininfo.flash_page_size as usize) {
        let mut xmodem = CRCu16::crc16xmodem();
        xmodem.digest(&page);

        binary_checksums.push(xmodem.get_crc());
    }

    //only check as many as our binary has
    assert_eq!(
        &binary_checksums[..binary_checksums.len()],
        &device_checksums[..binary_checksums.len()]
    );
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
