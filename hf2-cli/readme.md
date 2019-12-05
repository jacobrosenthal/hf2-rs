# hf2-cli
Replaces the cargo build command to include flashing over usb to connectd uf2 devices using [hf2 flashing over hid protocol](https://github.com/jacobrosenthal/hf2-rs).

## install
`cargo install hf2-cli`

On linux if building libusb fails you can also try setting up the native `libusb` library where it can be found by `pkg-config` or `vcpkg`.

## use
```
hf2 0.1.0
Microsoft HID Flashing Format

USAGE:
    hf2 [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -p <pid>        
    -v <vid>        

SUBCOMMANDS:
    bininfo                  This command states the current mode of the device
    dmesg                    Return internal log buffer if any. The result is a character array.
    flash                    flash
    help                     Prints this message or the help of the given subcommand(s)
    info                     Various device information. The result is a character array. See INFO_UF2.TXT in UF2
                             format for details.
    reset-into-app           Reset the device into user-space app.
    reset-into-bootloader    Reset the device into bootloader, usually for flashing
    verify                   verify
```
It will attempt to autodetect a device by sending the bininfo command any hid devices it finds and using the first one that responds. I don't think that should be destructive, but you can also specify pid and vid (before the command for some reason..) instead.

```
hf2 -v 0x239a -p 0x003d flash -f neopixel_rainbow.bin -a 0x4000
```
If you find an error, be sure to run with debug to see where in the process it failed
```
RUST_LOG=debug hf2 -v 0x239a -p 0x003d flash -f neopixel_rainbow.bin -a 0x4000
```
