use crate::command::{rx, xmit, Command, Commander, Error, NoResult};
use scroll::{ctx, ctx::TryIntoCtx, Pwrite, LE};

///Write a single page of flash memory. No Result.
pub struct WriteFlashPage {
    pub target_address: u32,
    pub data: Vec<u8>,
}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for &'a WriteFlashPage {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;

        dst.gwrite_with(self.target_address, &mut offset, LE)?;

        for i in &self.data {
            dst.gwrite_with(i, &mut offset, ctx)?;
        }

        Ok(offset)
    }
}

impl<'a> Commander<'a, NoResult> for WriteFlashPage {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let mut data = vec![0_u8; self.data.len() + 4];
        self.try_into_ctx(&mut data, LE)?;

        let command = Command::new(0x0006, 0, data);

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResult {})
    }
}
