use crate::command::{send, Commander, Error};
use scroll::{ctx, Pwrite};

/// Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
#[derive(Debug, Clone, Copy)]
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

impl<'a> Commander<'a, ReadWordsResponse<'a>> for ReadWords {
    const ID: u32 = 0x0008;
    const RESPONSE: bool = true;
    const RESULT: bool = true;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<ReadWordsResponse<'a>>, Error> {
    //     send(*self, data, d)
    // }
}

#[derive(Debug, PartialEq)]
pub struct ReadWordsResponse<'a> {
    pub words: &'a [u8],
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ReadWordsResponse<'a> {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], _le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 4 {
            return Err(Error::Parse);
        }

        let offset = 0;

        Ok((ReadWordsResponse { words: this }, offset))
    }
}
