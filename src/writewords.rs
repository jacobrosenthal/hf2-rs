use crate::command::{send, Commander, Error, NoResponse};
use scroll::{ctx::TryIntoCtx, Pwrite};

/// Dual of READ WORDS, with the same constraints.
#[derive(Debug, Clone, Copy)]
pub struct WriteWords<'a> {
    pub target_address: u32,
    pub num_words: u32,
    pub words: &'a [u8],
}

impl<'a> TryIntoCtx<::scroll::Endian> for &'a WriteWords<'a> {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, ctx)?;
        dst.gwrite_with(self.num_words, &mut offset, ctx)?;

        for i in self.words {
            dst.gwrite_with(i, &mut offset, ctx)?;
        }

        Ok(offset)
    }
}

impl<'a> Commander<'a, NoResponse> for WriteWords<'a> {
    const ID: u32 = 0x0009;
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
