use crate::command::{rx, xmit, Command, CommandResponseStatus, Commander, Error};
use scroll::{ctx, ctx::TryIntoCtx, Pread, Pwrite, LE};

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

impl<'a> Commander<'a, ChksumPagesResult> for ChksumPages {
    fn send(&self, d: &hidapi::HidDevice) -> Result<ChksumPagesResult, Error> {
        let mut data = vec![0_u8; 8];
        self.try_into_ctx(&mut data, LE)?;

        let command = Command::new(0x0007, 0, data.to_vec());

        xmit(command, d)?;

        let rsp = rx(d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::MalformedRequest);
        }

        let res: ChksumPagesResult =
            (rsp.data.as_slice()).pread_with::<ChksumPagesResult>(0, LE)?;

        Ok(res)
    }
}

///Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
#[derive(Debug, PartialEq)]
pub struct ChksumPagesResult {
    pub chksums: Vec<u16>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ChksumPagesResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut chksums: Vec<u16> = vec![0; this.len() / 2];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut chksums, le)?;

        Ok((ChksumPagesResult { chksums }, offset))
    }
}
