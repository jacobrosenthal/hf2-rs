use crate::command::Response;
use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use core::convert::From;
use core::convert::TryFrom;
use core::convert::TryInto;
use scroll::{ctx, ctx::TryIntoCtx, Pwrite, LE};

///Compute checksum of a number of pages. The checksum algorithm used is CRC-16-CCITT.
pub struct ChksumPages {
    pub target_address: u32,
    pub num_pages: u32,
}

//todo, until to_bytes not nightly
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

impl<'a> Commander<'a> for ChksumPages {
    fn send(&self, mut data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response<'a>, Error> {
        let _ = self.try_into_ctx(&mut data, LE)?;

        xmit(0x0007, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        Ok(Response::ChksumPages(ChksumPagesResponse::from(rsp.data)))
    }
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

// impl<'a> TryFrom<&'a [u8]> for ChksumPagesResponse<'a> {
//     type Error = Error;

//     fn try_from(this: &'a [u8]) -> Result<ChksumPagesResponse<'a>, Self::Error> {
//         if this.len() < 2 {
//             return Err(Error::Parse);
//         }

//         Ok(ChksumPagesResponse { chksums: this })
//     }
// }

impl<'a> core::convert::From<&'a [u8]> for ChksumPagesResponse<'a> {
    fn from(this: &'a [u8]) -> ChksumPagesResponse<'a> {
        // if this.len() < 2 {
        //     return Err(Error::Parse);
        // }

        ChksumPagesResponse { chksums: this }
    }
}
