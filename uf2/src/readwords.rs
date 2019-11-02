use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use crate::mock::HidMockable;
use core::convert::TryInto;
use scroll::{ctx, ctx::TryIntoCtx, Pread, Pwrite, LE};

/// Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub struct ReadWords {
    pub target_address: u32,
    pub num_words: u32,
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

    fn send<T: HidMockable>(
        &self,
        mut data: &'a mut [u8],
        d: &T,
    ) -> Result<ReadWordsResponse<'a>, Error> {
        let offset = self.try_into_ctx(&mut data, LE)?;

        xmit(Self::ID, 0, &data[0..offset], d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        let res: ReadWordsResponse = rsp.data.pread_with::<ReadWordsResponse>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub struct ReadWordsResponse<'a> {
    words: &'a [u8],
}

impl<'a> ReadWordsResponse<'a> {
    pub fn iter(&'a self) -> impl 'a + Iterator<Item = u32> {
        self.words.chunks_exact(4).map(|chunk| {
            //no panic, chunks exact is always &[u8; 2]
            let blah: &[u8; 4] = chunk.try_into().unwrap();
            u32::from_le_bytes(*blah)
        })
    }
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
