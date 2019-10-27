use crate::command::{send, Commander, Error, NoResponse};
use scroll::ctx;

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
#[derive(Debug, Clone, Copy)]
pub struct StartFlash {}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for StartFlash {
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

impl<'a> Commander<'a, NoResponse> for StartFlash {
    const ID: u32 = 0x0005;
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
