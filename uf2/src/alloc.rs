use crate::bininfo::{BinInfo, BinInfoMode, BinInfoResponse};
use crate::chksumpages::ChksumPages;
use crate::command::{Commander, Error};
use crate::dmesg::Dmesg;
use crate::info::Info;
use crate::mock::HidMockable;
use crate::readwords::ReadWords;
use crate::resetintoapp::ResetIntoApp;
use crate::resetintobootloader::ResetIntoBootloader;
use crate::startflash::StartFlash;
use crate::writeflashpage::WriteFlashPage;
use crate::writewords::WriteWords;
use crc::{self, crc16, Hasher16};

extern crate alloc;

use alloc::{string::String, string::ToString, vec, vec::Vec};

/// Reset the device into user-space app.
pub fn reset_into_app(d: &hidapi::HidDevice) -> Result<(), Error> {
    let _ = ResetIntoApp {}.send(&mut [], d)?;
    Ok(())
}

/// Reset the device into bootloader, usually for flashing.
pub fn reset_into_bootloader(d: &hidapi::HidDevice) -> Result<(), Error> {
    let _ = ResetIntoBootloader {}.send(&mut [], d)?;
    Ok(())
}

/// Various device information. See INFO_UF2.TXT in UF2 format for details.
pub fn info(d: &hidapi::HidDevice) -> Result<String, Error> {
    let mut scratch = vec![0_u8; 128];

    let info = Info {}.send(&mut scratch, d)?;
    Ok(info.info.to_string())
}

/// This command states the current mode of the device.
pub fn bininfo(d: &hidapi::HidDevice) -> Result<BinInfoResponse, Error> {
    let mut scratch = vec![0_u8; 64];

    BinInfo {}.send(&mut scratch, d)
}

/// Return internal log buffer if any, and if supported.
pub fn dmesg(d: &hidapi::HidDevice) -> Result<String, Error> {
    let mut scratch = vec![0_u8; 64];

    // todo, test. not supported on my board
    let dmesg = Dmesg {}.send(&mut scratch, d)?;

    Ok(dmesg.logs.to_string())
}

/// Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
pub fn chksum_pages(
    target_address: u32,
    num_pages: u32,
    d: &hidapi::HidDevice,
) -> Result<Vec<u16>, Error> {
    let mut scratch = vec![0_u8; 64];

    let checksums = ChksumPages {
        target_address,
        num_pages,
    }
    .send(&mut scratch, d)?;

    Ok(checksums.iter().collect())
}

/// Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub fn read_words(
    target_address: u32,
    num_words: u32,
    d: &hidapi::HidDevice,
) -> Result<Vec<u32>, Error> {
    //todo calc size based on max message size?
    let mut scratch = vec![0_u8; 1024];

    let words_response = ReadWords {
        target_address,
        num_words,
    }
    .send(&mut scratch, d)?;

    Ok(words_response.iter().collect())
}

/// Dual of READ WORDS, with the same constraints.
pub fn write_words(
    target_address: u32,
    num_words: u32,
    words: &[u32],
    d: &hidapi::HidDevice,
) -> Result<(), Error> {
    let mut scratch = vec![0_u8; 1024];

    let _ = WriteWords {
        target_address,
        num_words,
        words,
    }
    .send(&mut scratch, d)?;

    Ok(())
}

/// Write a single page of flash memory.
pub fn write_flash_page(
    target_address: u32,
    data: &[u8],
    d: &hidapi::HidDevice,
) -> Result<(), Error> {
    let mut scratch = vec![0_u8; 1024];

    let _ = WriteFlashPage {
        target_address,
        data,
    }
    .send(&mut scratch, d)?;

    Ok(())
}

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
pub fn start_flash(d: &hidapi::HidDevice) -> Result<(), Error> {
    let mut scratch = vec![0_u8; 64];

    let _ = StartFlash {}.send(&mut scratch, d)?;
    Ok(())
}

/// Flash
pub fn flash(binary: &[u8], address: u32, d: &hidapi::HidDevice) -> Result<(), Error> {
    let mut scratch = vec![0_u8; 1024];

    let bininfo = BinInfo {}.send(&mut scratch, d)?;
    log::debug!("{:?}", bininfo);

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = StartFlash {}.send(&mut scratch, d)?;
    }

    //pad zeros to page size
    let padded_num_pages = (binary.len() as f64 / f64::from(bininfo.flash_page_size)).ceil() as u32;
    let padded_size = padded_num_pages * bininfo.flash_page_size;
    log::debug!(
        "binary is {} bytes, padding to {} bytes",
        binary.len(),
        padded_size
    );

    // get checksums of existing pages
    let top_address = address + padded_size as u32;
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
        let checksums = ChksumPages {
            target_address,
            num_pages,
        }
        .send(&mut scratch, d)?;

        for checksum in checksums.iter() {
            device_checksums.push(checksum)
        }
    }
    log::debug!("checksums received {:04X?}", device_checksums);

    // only write changed contents
    for (page_index, page) in binary.chunks(bininfo.flash_page_size as usize).enumerate() {
        let mut digest1 = crc16::Digest::new_custom(crc16::X25, 0u16, 0u16, crc::CalcType::Normal);

        //pad with zeros in case its last page and under size
        if (page.len() as u32) < bininfo.flash_page_size {
            let mut padded = page.to_vec();
            padded.resize(bininfo.flash_page_size as usize, 0);
            digest1.write(&padded);
        } else {
            digest1.write(&page);
        }

        if digest1.sum16() != device_checksums[page_index] {
            log::debug!(
                "ours {:04X?} != {:04X?} theirs, updating page {}",
                digest1.sum16(),
                device_checksums[page_index],
                page_index,
            );

            let target_address = address + bininfo.flash_page_size * page_index as u32;

            let _ = WriteFlashPage {
                target_address,
                data: page,
            }
            .send(&mut scratch, d)?;
        } else {
            log::debug!("not updating page {}", page_index,);
        }
    }

    let _ = ResetIntoApp {}.send(&mut [], d)?;
    Ok(())
}

/// Verify
pub fn verify(binary: &[u8], address: u32, d: &hidapi::HidDevice) -> Result<(), Error> {
    let mut scratch = vec![0_u8; 1024];

    let bininfo = BinInfo {}.send(&mut scratch, d)?;

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = StartFlash {}.send(&mut scratch, d)?;
    }

    //pad zeros to page size
    let padded_num_pages = (binary.len() as f64 / f64::from(bininfo.flash_page_size)).ceil() as u32;
    let padded_size = padded_num_pages * bininfo.flash_page_size;

    // get checksums of existing pages
    let top_address = address + padded_size as u32;
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
        let checksums = ChksumPages {
            target_address,
            num_pages,
        }
        .send(&mut scratch, d)?;

        for checksum in checksums.iter() {
            device_checksums.push(checksum)
        }
    }

    let mut binary_checksums = vec![];

    //collect and sums so we can view all mismatches, not just first
    for page in binary.chunks(bininfo.flash_page_size as usize) {
        let mut digest1 = crc16::Digest::new_custom(crc16::X25, 0u16, 0u16, crc::CalcType::Normal);
        digest1.write(&page);

        binary_checksums.push(digest1.sum16());
    }

    //only check as many as our binary has
    assert_eq!(
        &binary_checksums[..binary_checksums.len()],
        &device_checksums[..binary_checksums.len()]
    );

    Ok(())
}

#[cfg(feature = "hidapi")]
use hidapi::HidDevice;

#[cfg(feature = "hidapi")]
impl HidMockable for HidDevice {
    fn my_write(&self, data: &[u8]) -> Result<usize, Error> {
        self.write(data).map_err(core::convert::From::from)
    }
    fn my_read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        self.read(buf).map_err(core::convert::From::from)
    }
}
