# hf2
Implements [Microsofts HID Flashing Format (HF2)](https://github.com/microsoft/uf2/blob/86e101e3a282553756161fe12206c7a609975e70/hf2.md) to upload firmware to UF2 bootloaders. UF2 is factory programmed extensively by [Microsoft MakeCode](https://www.microsoft.com/en-us/makecode) and [Adafruit](https://www.adafruit.com) hardware.

## install and setup

Utilizes the [hidapi](https://crates.io/crates/hidapi) crate which doesnt appear to need any dependencies.

On linux if building libusb fails you can also try setting up the native `libusb` library where it can be found by `pkg-config` or `vcpkg`.

On mac, as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application. 

## use

```
let chk: ChksumPagesResponse = ChksumPages {
    0x4000,
    1,
}.send(&d)?;
println!(chk.chksums);
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
