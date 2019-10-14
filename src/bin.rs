use crc::{self, crc16, Hasher16};
use hidapi::{HidApi, HidDevice};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;
use uf2::*;

fn main() -> Result<(), Error> {
    let args = Opt::from_args();

    let api = HidApi::new().expect("Couldn't find system usb");

    let d = if let (Some(v), Some(p)) = (args.vid, args.pid) {
        api.open(v, p)
            .expect("Are you sure device is plugged in and in uf2 mode?")
    } else {
        println!("no vid/pid provided..");

        let mut device: Option<HidDevice> = None;

        for device_info in api.devices() {
            println!(
                "trying {:?} {:?}",
                device_info
                    .manufacturer_string
                    .as_ref()
                    .unwrap_or(&"".to_string()),
                device_info
                    .product_string
                    .as_ref()
                    .unwrap_or(&"".to_string())
            );

            if let Ok(d) = device_info.open_device(&api) {
                let info = Info {}.send(&d);
                if info.is_ok() {
                    device = Some(d);
                    break;
                }
            }
        }
        device.expect("Are you sure device is plugged in and in uf2 mode?")
    };

    match args.cmd {
        Cmd::resetIntoApp => reset_into_bootloader(&d)?,
        Cmd::resetIntoBootloader => reset_into_app(&d)?,
        Cmd::info => info(&d)?,
        Cmd::bininfo => bininfo(&d)?,
        Cmd::dmesg => dmesg(&d)?,
        Cmd::flash { file, address } => flash(file, address, &d)?,
        Cmd::verify { file, address } => verify(file, address, &d)?,
    }

    Ok(())
}

fn reset_into_app(d: &HidDevice) -> Result<(), Error> {
    let _ = ResetIntoApp {}.send(&d)?;
    Ok(())
}

fn reset_into_bootloader(d: &HidDevice) -> Result<(), Error> {
    let _ = ResetIntoBootloader {}.send(&d);
    Ok(())
}

fn info(d: &HidDevice) -> Result<(), Error> {
    let info: InfoResult = Info {}.send(&d)?;
    println!("{:?}", info);
    Ok(())
}

fn bininfo(d: &HidDevice) -> Result<(), Error> {
    let bininfo: BinInfoResult = BinInfo {}.send(&d)?;
    println!(
        "{:?} {:?}kb",
        bininfo,
        bininfo.flash_num_pages * bininfo.flash_page_size / 1024
    );
    Ok(())
}

fn dmesg(d: &HidDevice) -> Result<(), Error> {
    // todo, test. not supported on my board
    let dmesg: DmesgResult = Dmesg {}.send(&d)?;
    println!("{:?}", dmesg);
    Ok(())
}

fn flash(file: PathBuf, address: u32, d: &HidDevice) -> Result<(), Error> {
    let bininfo: BinInfoResult = BinInfo {}.send(&d)?;

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = StartFlash {}.send(&d)?;
    }

    //shouldnt there be a chunking interator for this?
    let mut f = File::open(file)?;
    let mut binary = Vec::new();
    f.read_to_end(&mut binary)?;

    //pad zeros to page size
    let padded_num_pages = (binary.len() as f64 / f64::from(bininfo.flash_page_size)).ceil() as u32;
    let padded_size = padded_num_pages * bininfo.flash_page_size;
    for _i in 0..(padded_size as usize - binary.len()) {
        binary.push(0x0);
    }

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
        let chk: ChksumPagesResult = ChksumPages {
            target_address,
            num_pages,
        }
        .send(&d)?;
        device_checksums.extend_from_slice(&chk.chksums[..]);
    }

    // only write changed contents
    for (page_index, page) in binary.chunks(bininfo.flash_page_size as usize).enumerate() {
        let mut digest1 = crc16::Digest::new_custom(crc16::X25, 0u16, 0u16, crc::CalcType::Normal);
        digest1.write(&page);

        if digest1.sum16() != device_checksums[page_index] {
            let target_address = address + bininfo.flash_page_size * page_index as u32;
            let _ = WriteFlashPage {
                target_address,
                data: page.to_vec(),
            }
            .send(&d)?;
        }
    }

    println!("Success");
    let _ = ResetIntoApp {}.send(&d)?;
    Ok(())
}

fn verify(file: PathBuf, address: u32, d: &HidDevice) -> Result<(), Error> {
    let bininfo: BinInfoResult = BinInfo {}.send(&d)?;

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = StartFlash {}.send(&d)?;
    }

    //shouldnt there be a chunking interator for this?
    let mut f = File::open(file)?;
    let mut binary = Vec::new();
    f.read_to_end(&mut binary)?;

    //pad zeros to page size
    let padded_num_pages = (binary.len() as f64 / bininfo.flash_page_size as f64).ceil() as u32;
    let padded_size = padded_num_pages * bininfo.flash_page_size;
    for _i in 0..(padded_size as usize - binary.len()) {
        binary.push(0x0);
    }

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
        let chk: ChksumPagesResult = ChksumPages {
            target_address,
            num_pages,
        }
        .send(&d)?;
        device_checksums.extend_from_slice(&chk.chksums[..]);
    }

    let mut binary_checksums = vec![];

    for page in binary.chunks(bininfo.flash_page_size as usize) {
        let mut digest1 = crc16::Digest::new_custom(crc16::X25, 0u16, 0u16, crc::CalcType::Normal);
        digest1.write(&page);

        binary_checksums.push(digest1.sum16());
    }

    assert_eq!(
        &binary_checksums[..binary_checksums.len() - 1],
        &device_checksums[..binary_checksums.len() - 1]
    );
    println!("Success");

    Ok(())
}

fn parse_hex_32(input: &str) -> Result<u32, std::num::ParseIntError> {
    if input.starts_with("0x") {
        u32::from_str_radix(&input[2..], 16)
    } else {
        input.parse::<u32>()
    }
}
fn parse_hex_16(input: &str) -> Result<u16, std::num::ParseIntError> {
    if input.starts_with("0x") {
        u16::from_str_radix(&input[2..], 16)
    } else {
        input.parse::<u16>()
    }
}

#[allow(non_camel_case_types)]
#[derive(StructOpt, Debug, PartialEq)]
pub enum Cmd {
    ///Reset the device into user-space app.
    resetIntoApp,
    ///Reset the device into bootloader, usually for flashing
    resetIntoBootloader,

    /// Various device information. The result is a character array. See INFO_UF2.TXT in UF2 format for details.
    info,

    /// This command states the current mode of the device
    bininfo,

    ///Return internal log buffer if any. The result is a character array.
    dmesg,

    /// flash
    flash {
        #[structopt(short = "f")]
        file: PathBuf,
        #[structopt(short, parse(try_from_str = parse_hex_32))]
        address: u32,
    },

    /// verify
    verify {
        #[structopt(short = "f")]
        file: PathBuf,
        #[structopt(short, parse(try_from_str = parse_hex_32))]
        address: u32,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "uf2", about = "Microsoft HID Flashing Format")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,

    #[structopt(short, parse(try_from_str = parse_hex_16))]
    pid: Option<u16>,
    #[structopt(short, parse(try_from_str = parse_hex_16))]
    vid: Option<u16>,
}
