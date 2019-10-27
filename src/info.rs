use crate::command::{send, Commander, Error};
use scroll::{ctx, Pread, LE};

/// Various device information. See INFO_UF2.TXT in UF2 format for details.
#[derive(Debug, Clone, Copy)]
pub struct Info {}

impl<'a> ctx::TryIntoCtx<::scroll::Endian> for Info {
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

impl<'a> Commander<'a, InfoResponse<'a>> for Info {
    const ID: u32 = 0x0002;
    const RESPONSE: bool = true;
    const RESULT: bool = true;

    // fn send(
    //     &self,
    //     mut data: &'a mut [u8],
    //     d: &hidapi::HidDevice,
    // ) -> Result<Option<InfoResponse<'a>>, Error> {
    //     send(*self, data, d)
    // }
}

#[derive(Debug, PartialEq)]
pub struct InfoResponse<'a> {
    pub info: &'a str,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for InfoResponse<'a> {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], _le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let offset = 0;

        //u8, no endianness
        let info = core::str::from_utf8(&this)?;

        Ok((InfoResponse { info }, offset))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_response() {
        let data: Vec<u8> = vec![
            0x55, 0x46, 0x32, 0x20, 0x42, 0x6F, 0x6F, 0x74, 0x6C, 0x6F, 0x61, 0x64, 0x65, 0x72,
            0x20, 0x76, 0x33, 0x2E, 0x36, 0x2E, 0x30, 0x20, 0x53, 0x46, 0x48, 0x57, 0x52, 0x4F,
            0x0D, 0x0A, 0x4D, 0x6F, 0x64, 0x65, 0x6C, 0x3A, 0x20, 0x50, 0x79, 0x47, 0x61, 0x6D,
            0x65, 0x72, 0x0D, 0x0A, 0x42, 0x6F, 0x61, 0x72, 0x64, 0x2D, 0x49, 0x44, 0x3A, 0x20,
            0x53, 0x41, 0x4D, 0x44, 0x35, 0x31, 0x4A, 0x31, 0x39, 0x41, 0x2D, 0x50, 0x79, 0x47,
            0x61, 0x6D, 0x65, 0x72, 0x2D, 0x4D, 0x34, 0x0D, 0x0A,
        ];

        let info_result = InfoResponse {
info: "UF2 Bootloader v3.6.0 SFHWRO\r\nModel: PyGamer\r\nBoard-ID: SAMD51J19A-PyGamer-M4\r\n"
        };

        let res: InfoResponse = (data.as_slice()).pread_with::<InfoResponse>(0, LE).unwrap();

        assert_eq!(res, info_result);
    }
}
