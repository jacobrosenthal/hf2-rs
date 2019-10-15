use crate::command::{rx, xmit, Command, Commander, Error, NoResult};
use scroll::{ctx, ctx::TryIntoCtx, Pread, Pwrite, LE};

///Dual of READ WORDS, with the same constraints. No Result.
pub struct WriteWords {
    pub target_address: u32,
    pub num_words: u32,
    pub words: Vec<u32>,
}

impl<'a> TryIntoCtx<::scroll::Endian> for &'a WriteWords {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, ctx)?;
        dst.gwrite_with(self.num_words, &mut offset, ctx)?;

        for i in &self.words {
            dst.gwrite_with(i, &mut offset, ctx)?;
        }

        Ok(offset)
    }
}

impl<'a> Commander<'a, NoResult> for WriteWords {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let data = &mut [0_u8; 64];
        self.try_into_ctx(&mut data, LE)?;

        let command = Command::new(0x0009, 0, data.to_vec());

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResult {})
    }
}
