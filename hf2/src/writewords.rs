use crate::command::{rx, xmit, Command};
use crate::Error;
use scroll::Pwrite;

///Dual of READ WORDS, with the same constraints. Empty tuple response.
pub fn write_words(
    d: &hidapi::HidDevice,
    target_address: u32,
    num_words: u32,
    words: Vec<u32>,
) -> Result<(), Error> {
    let mut buffer = vec![0_u8; words.len() * 4 + 8];
    let mut offset = 0;

    buffer.gwrite_with(target_address, &mut offset, scroll::LE)?;
    buffer.gwrite_with(num_words, &mut offset, scroll::LE)?;
    for i in words {
        buffer.gwrite_with(i, &mut offset, scroll::LE)?;
    }

    xmit(Command::new(0x0009, 0, buffer), d)?;

    rx(d).map(|_| ())
}
