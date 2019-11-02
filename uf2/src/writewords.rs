use crate::command::{rx, xmit, Commander, Error, NoResponse};
use crate::mock::HidMockable;
use scroll::{ctx::TryIntoCtx, Pwrite, LE};

/// Dual of READ WORDS, with the same constraints.
#[derive(Debug, Clone, Copy)]
pub struct WriteWords<'a> {
    pub target_address: u32,
    pub num_words: u32,
    pub words: &'a [u32],
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

    fn send<T: HidMockable>(&self, mut data: &'a mut [u8], d: &T) -> Result<NoResponse, Error> {
        debug_assert!(data.len() >= self.words.len() * 4 + 8);

        let offset = self.try_into_ctx(&mut data, LE)?;

        xmit(Self::ID, 0, &data[0..offset], d)?;

        let _ = rx(data, d)?;

        Ok(NoResponse {})
    }
}
