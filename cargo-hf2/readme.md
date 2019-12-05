# cargo-hf2
Replaces the cargo build command to include flashing over usb to connectd uf2 devices using [hf2 flashing over hid protocol](https://github.com/jacobrosenthal/hf2-rs).

## install
`cargo install cargo-hf2`

On macOS, it doesnt seem to require any other packages. Note this protocol works over USB HID, which is an input standard, and as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application.

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

If you find an error, be sure to run with debug to see where in the process it failed `RUST_LOG=debug cargo hf2 --release --example ferris_img`