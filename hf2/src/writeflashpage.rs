use crate::command::{rx, xmit, Command, Commander, Error, NoResponse};
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

impl<'a> Commander<'a, NoResponse> for WriteFlashPage {
    const ID: u32 = 0x0006;

    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        let mut data = vec![0_u8; self.data.len() + 4];
        let _ = self.try_into_ctx(&mut data, LE)?;

        let command = Command::new(Self::ID, 0, data);

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResponse {})
    }
}
