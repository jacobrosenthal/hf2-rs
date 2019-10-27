use crate::command::{send, Commander, Error, NoResponse};
use scroll::ctx;

/// Reset the device into user-space app.
#[derive(Debug, Clone, Copy)]
pub struct ResetIntoApp {}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for ResetIntoApp {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        _dst: &mut [u8],
        _ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let offset = 0;

        Ok(offset)
    }
}

impl<'a> Commander<'a, NoResponse> for ResetIntoApp {
    const ID: u32 = 0x0003;
    const RESPONSE: bool = false;
    const RESULT: bool = false;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<NoResponse>, Error> {
    //     send(*self, data, d)
    // }
}
