# hf2
Implements [Microsofts HID Flashing Format (HF2)](https://github.com/microsoft/uf2/blob/86e101e3a282553756161fe12206c7a609975e70/hf2.md) as both a library and binary which uploads firmware to their UF2 bootloader. It is factory programmed by default extensively by [Microsoft MakeCode](https://www.microsoft.com/en-us/makecode) and [Adafruit](https://www.microsoft.com/en-us/makecode) hardware.

## install and setup

On macOS, it doesnt seem to require any other packages. Note this protocol works over USB HID, which is an input standard, and as of Catalina you will get a permissions prompt and must follow directions to allow "Input Monitoring" for the Terminal application.


```
let chk: ChksumPagesResponse = ChksumPages {
    0x4000,
    1,
}.send(&d)?;
println!(chk.chksums);
```

