# hf2-cli

Command line implementation of the [hf2 flashing over hid protocol](https://github.com/jacobrosenthal/hf2-rs/tree/master/hf2) commonly used in by [Microsoft MakeCode](https://www.microsoft.com/en-us/makecode) and [Adafruit](https://www.adafruit.com) hardware.

Unless you know otherwise, you probably want [cargo-hf2](https://github.com/jacobrosenthal/hf2-rs)

## prerequisites

Utilizes the [hidapi-sys crate](https://crates.io/crates/hidapi) which uses [libusb](https://github.com/libusb/hidapi).

### linux

Youll need libusb depending on your distro you might do `sudo apt-get install libudev-dev libusb-1.0-0-dev`.

If you'd like to not use sudo, you'll need udev rules. With your board plugged in and in bootloader mode, use `lsusb` to find your vendorid, seen here as 239a

```bash
Bus 001 Device 087: ID 239a:001b Adafruit Industries Feather M0
```

Then put your vendorid below and save to something like /etc/udev/rules.d/99-adafruit-boards.rules

```bash
ATTRS{idVendor}=="239a", ENV{ID_MM_DEVICE_IGNORE}="1"
SUBSYSTEM=="usb", ATTRS{idVendor}=="239a", MODE="0666"
SUBSYSTEM=="tty", ATTRS{idVendor}=="239a", MODE="0666"
```

Then reboot or run

```bash
sudo udevadm control --reload-rules
sudo udevadm trigger
```

### mac

On mac, as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application.

## install

`cargo install hf2-cli`

## `hf2` flashing elf files, or as a cargo runner

`cargo build --release --example blinky_basic` then `hf2 elf target/thumbv7em-none-eabihf/release/examples/blinky_basic`

Hf2 will attempt to autodetect a device by sending the bininfo command to any whitelisted vid/pids it finds connected and using the first one that responds, or you can specify pid and vid (before the subcommand) instead. `hf2 --vid 0x239a --pid 0x003d elf target/thumbv7em-none-eabihf/release/examples/blinky_basic`

However the optimal use is as a cargo runner. In your .cargo/config set hf2 as your runner

```toml
[target.thumbv7em-none-eabihf]
runner = "hf2 elf"
```

Then either `cargo run --release --example blinky_basic` Or use your ide's "run" button and it will build and upload.

## hf2 standalone to flash binaries

The flash command deals in binaries, not elf files so you're going to have to get a bin with something like [cargo binutils](https://github.com/rust-embedded/cargo-binutils) `cargo objcopy --release --example blinky_basic -- -O binary blinky_basic.bin`

Then all you need your bootloaders address offset. `hf2 blinky_basic.bin -a 0x4000`

Hf2 will attempt to autodetect a device by sending the bininfo command to any whitelisted vid/pids it finds connected and using the first one that responds, or you can specify pid and vid (before the subcommand) instead. `hf2 -v 0x239a -p 0x003d flash -f blinky_basic.bin -a 0x4000`

## troubleshooting

If it cant find a device, make sure your device is in a bootloader mode ready to receive firmware.

```bash
thread 'main' panicked at 'Are you sure device is plugged in and in bootloader mode?: OpenHidDeviceError', src/libcore/result.rs:1165:5
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace.
```

On the PyGamer, two button presses enables a blue and green screen that says PyGamer and also generally creates a flash drive which you should be able to see (though this doesn't use that method).

If you find another error, be sure to run with debug to see where in the process it failed and include those logs when reporting

```bash
RUST_LOG=debug hf2 -v 0x239a -p 0x003d flash -f neopixel_rainbow.bin -a 0x4000
```
