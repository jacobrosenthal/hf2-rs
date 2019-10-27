use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use scroll::{ctx, ctx::TryIntoCtx, Pread, Pwrite, LE};
use std::convert::TryInto;

///Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
pub struct ChksumPages {
    pub target_address: u32,
    pub num_pages: u32,
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for &'a ChksumPages {
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

    fn send(
        &self,
        mut data: &'a mut [u8],
        d: &hidapi::HidDevice,
    ) -> Result<ChksumPagesResponse<'a>, Error> {
        let _ = self.try_into_ctx(&mut data, LE)?;

        xmit(Self::ID, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        let res: ChksumPagesResponse = rsp.data.pread_with::<ChksumPagesResponse>(0, LE)?;

        Ok(res)
    }
}

///Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
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
