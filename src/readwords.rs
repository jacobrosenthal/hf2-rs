use crate::command::Response;
use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use core::convert::TryFrom;
use scroll::{ctx, ctx::TryIntoCtx, Pwrite, LE};

/// Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
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

impl<'a> Commander<'a> for ReadWords {
    fn send(&self, mut data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        let _ = self.try_into_ctx(&mut data, LE)?;

        xmit(0x0008, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }
        let resp = ReadWordsResponse::try_from(rsp.data)?;

        Ok(Response::ReadWords(resp))
    }
}

#[derive(Debug, PartialEq)]
pub struct ReadWordsResponse<'a> {
    pub words: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for ReadWordsResponse<'a> {
    type Error = Error;

    fn try_from(this: &'a [u8]) -> Result<ReadWordsResponse<'a>, Self::Error> {
        if this.len() < 4 {
            return Err(Error::Parse);
        }

        Ok(ReadWordsResponse { words: this })
    }
}
