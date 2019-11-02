# uf2-hid
Implements [Microsofts HID Flashing Format (HF2)](https://github.com/microsoft/uf2/blob/86e101e3a282553756161fe12206c7a609975e70/hf2.md) as both a library and binary.

## install and setup
Has built in support for usb using rusts [hidapi library](https://crates.io/crates/hidapi) and thus follows its requirements, or provides a HidMockable trait to bring your own.

On macOS, hidapi doesnt seem to require any other packages. Note this protocol works over USB HID, which is an input standard, and as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application.

On linux if building libusb fails you can also try setting up the native `libusb` library where it can be found by `pkg-config` or `vcpkg`.

## std usage
Provides trait for hidapi usage
```
use hidapi::HidApi;
use std::fs::File;
use std::io::Read;
use uf2::*;

fn main() -> Result<(), uf2::Error> {
    let api = HidApi::new().unwrap();
    let d = api.open(0x239A, 0x003D).unwrap();

    let mut f = File::open("./neopixel_rainbow.bin").unwrap();
    let mut binary = vec![];
    f.read_to_end(&mut binary).unwrap();

    flash(&binary, 0x4000, &d)?;
    Ok(())
}
```

## no_std
can be used on no_std without alloc with default-features disabled, but requires an implementation of the hid trait to write to, in the standard rust case hidapi, and a scratch buffer from somewhere, for brevity in this case, from a std vec. See the allocated helper functions but it looks something like:
```
let mut scratch = vec![0_u8; 64];

let bininfo = BinInfo {}.send(&mut scratch, &d)?;
println!("{:?}", bininfo);
```
The two reset commands, which have don't actually send or even attempt to receive data, can actually use a zero size scratch buffer.
```
let _ = ResetIntoApp {}.send(&mut [], &d)?;
```
Checksums are available as an iterator
```
let mut scratch = vec![0_u8; 1024];
let checksums = ChksumPages {
    target_address,
    num_pages,
}
.send(&mut scratch, &d)?;

for checksum in checksums.iter() {
    println!("{:?}", checksum);
}
```

## used via cli
```
uf2 0.1.0
Microsoft HID Flashing Format

USAGE:
    uf2 [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p, --pid <pid>
    -v, --vid <vid>

SUBCOMMANDS:
    bininfo                  This command states the current mode of the device.
    dmesg                    Return internal log buffer if any.
    flash                    Flash
    help                     Prints this message or the help of the given subcommand(s)
    info                     Various device information. See INFO_UF2.TXT in UF2 format for details.
    reset-into-app           Reset the device into user-space app.
    reset-into-bootloader    Reset the device into bootloader, usually for flashing. A BININFO command can be issued
                             to verify that.
    verify                   Verify
```
It will attempt to autodetect a device by sending the bininfo command any hid devices it finds and using the first one that responds. I don't think that should be destructive, but you can also specify pid and vid (before the command for some reason..) instead.

```
cargo run -- -v 0x0239 -p 0x003D flash -f neopixel_rainbow.bin -a 0x4000
```
If you find an error, be sure to run with debug to see where in the process it failed
```
RUST_LOG=debug cargo run -- -v 0x0239 -p 0x003D flash -f neopixel_rainbow.bin -a 0x4000
```
