# uf2-hid
Implements the hid uploader described at [Microsofts uf2 bootloader](https://github.com/microsoft/uf2/blob/86e101e3a282553756161fe12206c7a609975e70/hf2.md) as both a library and binary.

used as a library
```
let chk: ChksumPagesResult = ChksumPages {
    0x4000,
    1,
}.send(&d)?;

```
or via cli
```
uf2 0.1.0
Microsoft HID Flashing Format

USAGE:
    uf2 -p <pid> -v <vid> <SUBCOMMAND>

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
```
cargo run -- -v 9114 -p 61 flash -f neopixel_rainbow.bin -a 0x4000
```