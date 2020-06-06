use crate::command::{rx, xmit, Command};
use crate::Error;
use scroll::Pwrite;

///Write a single page of flash memory. Empty tuple response.
pub fn write_flash_page(
    d: &hidapi::HidDevice,
    target_address: u32,
    data: Vec<u8>,
) -> Result<(), Error> {
    let mut buffer = vec![0_u8; data.len() + 4];
    let mut offset = 0;

    buffer.gwrite_with(target_address, &mut offset, scroll::LE)?;
    for i in &data {
        buffer.gwrite_with(i, &mut offset, scroll::LE)?;
    }

    xmit(Command::new(0x0006, 0, buffer), d)?;

    rx(d).map(|_| ())
}
