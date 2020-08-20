use hf2::utils::{elf_to_bin, flash_bin, vendor_map, verify_bin};
use hidapi::{HidApi, HidDevice};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use structopt::StructOpt;

fn main() {
    pretty_env_logger::init();

    let args = Opt::from_args();

    let api = HidApi::new().expect("Couldn't find system usb");

    let d = if let (Some(v), Some(p)) = (args.vid, args.pid) {
        api.open(v, p)
            .expect("Are you sure device is plugged in and in bootloader mode?")
    } else {
        println!("no vid/pid provided..");

        let mut device: Option<HidDevice> = None;

        let vendor = vendor_map();

        for device_info in api.device_list() {
            if let Some(products) = vendor.get(&device_info.vendor_id()) {
                if products.contains(&device_info.product_id()) {
                    if let Ok(d) = device_info.open_device(&api) {
                        device = Some(d);
                        break;
                    }
                }
            }
        }
        device.expect("Are you sure device is plugged in and in bootloader mode?")
    };

    println!(
        "found {:?} {:?}",
        d.get_manufacturer_string(),
        d.get_product_string()
    );

    match args.cmd {
        Cmd::resetIntoApp => hf2::reset_into_app(&d).unwrap(),
        Cmd::resetIntoBootloader => hf2::reset_into_bootloader(&d).unwrap(),
        Cmd::info => info(&d),
        Cmd::bininfo => bininfo(&d),
        Cmd::dmesg => dmesg(&d),
        Cmd::flash { file, address } => {
            let binary = get_binary(file);
            let bininfo = hf2::bin_info(&d).expect("bin_info failed");
            log::debug!("{:?}", bininfo);

            flash_bin(&binary, address, &bininfo, &d).unwrap();
            println!("Success")
        }
        Cmd::verify { file, address } => {
            let binary = get_binary(file);
            let bininfo = hf2::bin_info(&d).expect("bin_info failed");
            log::debug!("{:?}", bininfo);

            verify_bin(&binary, address, &bininfo, &d).unwrap();
            println!("Success")
        }
        Cmd::elf { path } => {
            let (binary, address) = elf_to_bin(path).unwrap();

            let bininfo = hf2::bin_info(&d).expect("bin_info failed");
            log::debug!("{:?}", bininfo);

            flash_bin(&binary, address, &bininfo, &d).unwrap();
        }
    }
}

fn info(d: &HidDevice) {
    let info = hf2::info(&d).expect("info failed");
    println!("{:?}", info);
}

fn bininfo(d: &HidDevice) {
    let bininfo = hf2::bin_info(&d).expect("bin_info failed");
    println!(
        "{:?} {:?}kb",
        bininfo,
        bininfo.flash_num_pages * bininfo.flash_page_size / 1024
    );
}

fn dmesg(d: &HidDevice) {
    // todo, test. not supported on my board
    let dmesg = hf2::dmesg(&d).expect("dmesg failed");
    println!("{:?}", dmesg);
}

fn get_binary(file: PathBuf) -> Vec<u8> {
    //shouldnt there be a chunking interator for this?
    let mut f = File::open(file).unwrap();
    let mut binary = Vec::new();
    f.read_to_end(&mut binary).unwrap();
    binary
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

    /// flash binary, note includes a verify and reset into app
    flash {
        #[structopt(short = "f", name = "file", long = "file")]
        file: PathBuf,
        #[structopt(short = "a", name = "address", long = "address", parse(try_from_str = parse_hex_32))]
        address: u32,
    },

    /// verify binary
    verify {
        #[structopt(short = "f", name = "file", long = "file")]
        file: PathBuf,
        #[structopt(short = "a", name = "address", long = "address", parse(try_from_str = parse_hex_32))]
        address: u32,
    },

    /// flash elf, note includes a verify and reset into app
    elf {
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

#[derive(Debug, StructOpt)]
#[structopt(name = "hf2", about = "Microsoft HID Flashing Format")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Cmd,

    #[structopt(short = "p", name = "pid", long = "pid", parse(try_from_str = parse_hex_16))]
    pid: Option<u16>,
    #[structopt(short = "v", name = "vid", long = "vid", parse(try_from_str = parse_hex_16))]
    vid: Option<u16>,
}
