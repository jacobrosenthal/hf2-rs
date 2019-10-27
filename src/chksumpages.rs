use crate::command::{send, Commander, Error};
use core::convert::TryInto;
use scroll::{ctx, Pwrite};

///Compute checksum of a number of pages. The checksum algorithm used is CRC-16-CCITT.
#[derive(Debug, Clone, Copy)]
pub struct ChksumPages {
    pub target_address: u32,
    pub num_pages: u32,
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for ChksumPages {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, ctx)?;
        dst.gwrite_with(self.num_pages, &mut offset, ctx)?;

        Ok(offset)
    }
}

impl<'a> Commander<'a, ChksumPagesResponse<'a>> for ChksumPages {
    const ID: u32 = 0x0007;
    const RESPONSE: bool = true;
    const RESULT: bool = true;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<ChksumPagesResponse<'a>>, Error> {
    //     send(*self, data, d)
    // }
}

/// Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
#[derive(Debug, PartialEq)]
pub struct ChksumPagesResponse<'a> {
    chksums: &'a [u8],
}

impl<'a> ChksumPagesResponse<'a> {
    pub fn iter(&'a self) -> impl 'a + Iterator<Item = u16> {
        self.chksums.chunks_exact(2).map(|chunk| {
            //no panic, chunks exact is always &[u8; 2]
            let blah: &[u8; 2] = chunk.try_into().unwrap();
            u16::from_le_bytes(*blah)
        })
    }
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ChksumPagesResponse<'a> {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], _le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 2 {
            return Err(Error::Parse);
        }

        let offset = 0;

        Ok((ChksumPagesResponse { chksums: this }, offset))
    }
}
