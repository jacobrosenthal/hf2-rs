# hf2-cli
Command line implementation of the [hf2 flashing over hid protocol](https://github.com/jacobrosenthal/hf2-rs/tree/hf2) commonly used in by [Microsoft MakeCode](https://www.microsoft.com/en-us/makecode) and [Adafruit](https://www.microsoft.com/en-us/makecode) hardware.

## install
`cargo install hf2-cli`

Utilizes the [hidapi](https://crates.io/crates/hidapi) crate which doesnt appear to need any dependencies.

On linux if building libusb fails you can also try setting up the native `libusb` library where it can be found by `pkg-config` or `vcpkg`.

On mac, as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application. 

## use
```
$ hf2
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
It will attempt to autodetect a device by sending the bininfo command any whitelisted devices it finds and using the first one that responds or you can specify pid and vid (before the subcommand) instead.
```
hf2 -v 0x239a -p 0x003d flash -f neopixel_rainbow.bin -a 0x4000
```

## troubleshooting

If it cant find a device, make sure your device is in a bootloader mode ready to receive firmware.
```
thread 'main' panicked at 'Are you sure device is plugged in and in bootloader mode?: OpenHidDeviceError', src/libcore/result.rs:1165:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```
On the PyGamer, 2 button presses enables a blue and green screen that says PyGamer and also generally creates a flash drive which you should be able to see (though this doesn't use that method).

If you find another error, be sure to run with debug to see where in the process it failed and include those logs when reporting
```
RUST_LOG=debug hf2 -v 0x239a -p 0x003d flash -f neopixel_rainbow.bin -a 0x4000
```
