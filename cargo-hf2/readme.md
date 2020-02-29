# cargo-hf2
Replaces the cargo build command to include flashing over usb to connected uf2 devices using  [hf2 flashing over hid protocol](https://github.com/jacobrosenthal/hf2-rs/tree/master/hf2).

## prerequisites
Utilizes the [hidapi-sys crate](https://crates.io/crates/hidapi) which uses [libusb](https://github.com/libusb/hidapi).

### linux
Youll need libusb depending on your distro you might do `sudo apt-get install libudev-dev libusb-1.0-0-dev`.

If you'd like to not use sudo, you'll need udev rules. With your board plugged in and in bootloader mode, use `lsusb` to find your vendorid, seen here as 239a
```
Bus 001 Device 087: ID 239a:001b Adafruit Industries Feather M0
```
Then put your vendorid below and save to something like /etc/udev/rules.d/99-adafruit-boards.rules
```
ATTRS{idVendor}=="239a", ENV{ID_MM_DEVICE_IGNORE}="1"
SUBSYSTEM=="usb", ATTRS{idVendor}=="239a", MODE="0666"
SUBSYSTEM=="tty", ATTRS{idVendor}=="239a", MODE="0666"
```
Then reboot or run
```
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### mac
On mac, as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application. 

## install
`cargo install cargo-hf2`

## use
From a firmware directory you can run all the usual cargo build commands, --example and --release, with build replaced by hf2. Assuming the builds succeeds we open the usb device using a hardcoded whitelist and copy the file over.
```bash
$ cargo hf2 --example ferris_img --release --pid 0x003d --vid 0x239a
    Finished release [optimized + debuginfo] target(s) in 0.28s
    Flashing "./target/thumbv7em-none-eabihf/release/examples/ferris_img"
Success
    Finished in 0.037s
```
Optionally you can leave off pid and vid and it'll attempt to query any hid devices with the bininfo packet and write to the first one that responds
```bash
$ cargo hf2 --example ferris_img --release
    Finished release [optimized + debuginfo] target(s) in 0.24s
no vid/pid provided..
trying "" "Apple Internal Keyboard / Trackpad"
trying "Adafruit Industries" "PyGamer"
    Flashing "./target/thumbv7em-none-eabihf/release/examples/ferris_img"
Success
    Finished in 0.034s
```
If it cant find a device, make sure your device is in a bootloader mode. On the PyGamer, 2 button presses enables a blue and green screen that says PyGamer.
```bash
$ cargo hf2 --example ferris_img --release
    Finished release [optimized + debuginfo] target(s) in 0.20s
no vid/pid provided..
trying "" "Apple Internal Keyboard / Trackpad"
trying "" "Keyboard Backlight"
trying "" "Apple Internal Keyboard / Trackpad"
trying "" "Apple Internal Keyboard / Trackpad"
thread 'main' panicked at 'Are you sure device is plugged in and in bootloader mode?', src/libcore/option.rs:1166:5

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
