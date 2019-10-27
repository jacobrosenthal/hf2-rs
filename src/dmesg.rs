use crate::command::{send, Commander, Error};
use scroll::ctx;

/// Return internal log buffer if any.
#[derive(Debug, Clone, Copy)]
pub struct Dmesg {}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for Dmesg {
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

impl<'a> Commander<'a, DmesgResponse<'a>> for Dmesg {
    const ID: u32 = 0x0010;
    const RESPONSE: bool = true;
    const RESULT: bool = true;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<DmesgResponse<'a>>, Error> {
    //     send(*self, data, d)
    // }
}
#[derive(Debug, PartialEq)]
pub struct DmesgResponse<'a> {
    pub logs: &'a str,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for DmesgResponse<'a> {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], _le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = 0;

        //u8, no endianness
        let logs = core::str::from_utf8(&this)?;

        Ok((DmesgResponse { logs }, offset))
    }
}
