use crate::command::{send, Commander, Error, NoResponse};
use scroll::{ctx, Pwrite, LE};

/// Write a single page of flash memory.
#[derive(Debug, Clone, Copy)]
pub struct WriteFlashPage<'a> {
    pub target_address: u32,
    pub data: &'a [u8],
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for WriteFlashPage<'a> {
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
    const RESPONSE: bool = true;
    const RESULT: bool = false;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<NoResponse>, Error> {
    //     send(*self, data, d)
    // }
}
