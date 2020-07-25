use crate::command::{rx, xmit, Command, CommandResponse, CommandResponseStatus};
use crate::Error;
use scroll::{ctx, Pread, Pwrite, LE};

///Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub fn read_words(
    d: &hidapi::HidDevice,
    target_address: u32,
    num_words: u32,
) -> Result<ReadWordsResponse, Error> {
    let mut buffer = vec![0_u8; 8];
    let mut offset = 0;

    buffer.gwrite_with(target_address, &mut offset, scroll::LE)?;
    buffer.gwrite_with(num_words, &mut offset, scroll::LE)?;

    xmit(Command::new(0x0008, 0, buffer), d)?;

    match rx(d) {
        Ok(CommandResponse {
            status: CommandResponseStatus::Success,
            data,
            ..
        }) => (data.as_slice()).pread_with(0, LE),
        Ok(_) => Err(Error::CommandNotRecognized),
        Err(e) => Err(e),
    }
}

///Response to the read_words command
#[derive(Debug, PartialEq)]
pub struct ReadWordsResponse {
    pub words: Vec<u32>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ReadWordsResponse {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 4 {
            return Err(Error::Parse);
        }

        let mut words: Vec<u32> = vec![0; this.len() / 4];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut words, le)?;

        Ok((ReadWordsResponse { words }, offset))
    }
}
