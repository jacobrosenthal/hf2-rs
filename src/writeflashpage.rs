use crate::command::{rx, xmit, Commander, Error, NoResponse};
use scroll::{ctx, ctx::TryIntoCtx, Pwrite, LE};

/// Write a single page of flash memory.
#[derive(Debug, Clone, Copy)]
pub struct WriteFlashPage<'a> {
    pub target_address: u32,
    pub data: &'a [u8],
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for &'a WriteFlashPage<'a> {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, LE)?;

        for i in self.data {
            dst.gwrite_with(i, &mut offset, ctx)?;
        }

        Ok(offset)
    }
}

impl<'a> Commander<'a, NoResponse> for WriteFlashPage<'a> {
    const ID: u32 = 0x0006;

    fn send(&self, mut data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        debug_assert!(data.len() >= self.data.len() + 4);

        let _ = self.try_into_ctx(&mut data, LE)?;

        xmit(Self::ID, 0, &data, d)?;

        let _ = rx(data, d)?;

        Ok(NoResponse {})
    }
}
