use crate::command::{rx, xmit, Command, CommandResponseStatus, Commander, Error};
use scroll::{ctx, ctx::TryIntoCtx, Pread, Pwrite, LE};

///Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub struct ReadWords {
    target_address: u32,
    num_words: u32,
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for &'a ReadWords {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, ctx)?;
        dst.gwrite_with(self.num_words, &mut offset, ctx)?;

        Ok(offset)
    }
}

impl<'a> Commander<'a, ReadWordsResult> for ReadWords {
    const ID: u32 = 0x0008;

    fn send(&self, d: &hidapi::HidDevice) -> Result<ReadWordsResult, Error> {
        let mut data = vec![0_u8; 8];
        let _ = self.try_into_ctx(&mut data, LE)?;

        let command = Command::new(Self::ID, 0, data);

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
        if this.len() < 4 {
            return Err(Error::Parse);
        }

        let mut words: Vec<u32> = vec![0; this.len() / 4];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut words, le)?;

        Ok((ReadWordsResult { words }, offset))
    }
}
