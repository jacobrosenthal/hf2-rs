use crate::command::{rx, xmit, Command, CommandResponseStatus, Commander, Error};
use scroll::{ctx, Pread, Pwrite, LE};

///Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub struct ReadWords {
    target_address: u32,
    num_words: u32,
}

impl<'a> Commander<'a, ReadWordsResult> for ReadWords {
    fn send(&self, d: &hidapi::HidDevice) -> Result<ReadWordsResult, Error> {
        let data = &mut [0_u8; 8];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;
        data.gwrite_with(self.num_words, &mut offset, LE)?;

        let command = Command::new(0x0008, 0, data.to_vec());

        xmit(command, d)?;

        let rsp = rx(d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::MalformedRequest);
        }

        let res: ReadWordsResult = (rsp.data.as_slice()).pread_with::<ReadWordsResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub struct ReadWordsResult {
    pub words: Vec<u32>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ReadWordsResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut words: Vec<u32> = vec![0; this.len() / 4];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut words, le)?;

        Ok((ReadWordsResult { words }, offset))
    }
}
