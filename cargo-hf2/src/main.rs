use colored::*;
use hf2::utils::{elf_to_bin, flash_bin, vendor_map};
use hidapi::{HidApi, HidDevice};
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Instant;
use structopt::StructOpt;

fn main() {
    // Initialize the logging backend.
    pretty_env_logger::init();

    // Get commandline options.
    // Skip the first arg which is the calling application name.
    let opt = Opt::from_iter(std::env::args().skip(1));

    // Try and get the cargo project information.
    let project = cargo_project::Project::query(".").expect("Couldn't parse the Cargo.toml");

    // Decide what artifact to use.
    let artifact = if let Some(bin) = &opt.bin {
        cargo_project::Artifact::Bin(bin)
    } else if let Some(example) = &opt.example {
        cargo_project::Artifact::Example(example)
    } else {
        cargo_project::Artifact::Bin(project.name())
    };

    // Decide what profile to use.
    let profile = if opt.release {
        cargo_project::Profile::Release
    } else {
        cargo_project::Profile::Dev
    };

    // Try and get the artifact path.
    let path = project
        .path(
            artifact,
            profile,
            opt.target.as_deref(),
            "x86_64-unknown-linux-gnu",
        )
        .expect("Couldn't find the build result");

    // Remove first two args which is the calling application name and the `hf2` command from cargo.
    let mut args: Vec<_> = std::env::args().skip(2).collect();

    // todo, keep as iter. difficult because we want to filter map remove two items at once.
    // Remove our args as cargo build does not understand them.
    let flags = ["--pid", "--vid"].iter();
    for flag in flags {
        if let Some(index) = args.iter().position(|x| x == flag) {
            args.remove(index);
            args.remove(index);
        }
    }

    let status = Command::new("cargo")
        .arg("build")
        .args(args)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    if !status.success() {
        exit_with_process_status(status)
    }

    let api = HidApi::new().expect("Couldn't find system usb");

    let d = if let (Some(v), Some(p)) = (opt.vid, opt.pid) {
        api.open(v, p)
            .expect("Are you sure device is plugged in and in bootloader mode?")
    } else {
        println!(
            "    {} for a connected device with known vid/pid pair.",
            "Searching".green().bold(),
        );

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
        "    {} {:?} {:?}",
        "Trying ".green().bold(),
        d.get_manufacturer_string(),
        d.get_product_string()
    );

    println!("    {} {:?}", "Flashing".green().bold(), path);

    let (binary, address) = elf_to_bin(path).unwrap();

    // Start timer.
    let instant = Instant::now();

    let bininfo = hf2::bin_info(&d).expect("bin_info failed");
    log::debug!("{:?}", bininfo);

    flash_bin(&binary, address, &bininfo, &d).unwrap();

    // Stop timer.
    let elapsed = instant.elapsed();
    println!(
        "    {} in {}s",
        "Finished".green().bold(),
        elapsed.as_millis() as f32 / 1000.0
    );
}

#[cfg(unix)]
fn exit_with_process_status(status: std::process::ExitStatus) -> ! {
    use std::os::unix::process::ExitStatusExt;
    let status = status.code().or_else(|| status.signal()).unwrap_or(1);
    std::process::exit(status)
}

#[cfg(not(unix))]
fn exit_with_process_status(status: std::process::ExitStatus) -> ! {
    let status = status.code().unwrap_or(1);
    std::process::exit(status)
}

fn parse_hex_16(input: &str) -> Result<u16, std::num::ParseIntError> {
    if let Some(stripped) = input.strip_prefix("0x") {
        u16::from_str_radix(stripped, 16)
    } else {
        input.parse::<u16>()
    }
}

#[derive(Debug, StructOpt)]
struct Opt {
    // `cargo build` arguments
    #[structopt(name = "binary", long = "bin")]
    bin: Option<String>,
    #[structopt(name = "example", long = "example")]
    example: Option<String>,
    #[structopt(name = "package", short = "p", long = "package")]
    package: Option<String>,
    #[structopt(name = "release", long = "release")]
    release: bool,
    #[structopt(name = "target", long = "target")]
    target: Option<String>,
    #[structopt(name = "PATH", long = "manifest-path", parse(from_os_str))]
    manifest_path: Option<PathBuf>,
    #[structopt(long)]
    no_default_features: bool,
    #[structopt(long)]
    all_features: bool,
    #[structopt(long)]
    features: Vec<String>,

    #[structopt(name = "pid", long = "pid", parse(try_from_str = parse_hex_16))]
    pid: Option<u16>,
    #[structopt(name = "vid", long = "vid",  parse(try_from_str = parse_hex_16))]
    vid: Option<u16>,
    #[structopt(short = "Z")]
    unstable_features: Option<Vec<String>>
}
