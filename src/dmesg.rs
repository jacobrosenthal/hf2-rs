use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use scroll::{ctx, Pread, LE};

/// Return internal log buffer if any.
pub struct Dmesg {}

impl<'a> Commander<'a, DmesgResponse<'a>> for Dmesg {
    const ID: u32 = 0x0010;

    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<DmesgResponse<'a>, Error> {
        xmit(Self::ID, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        let res: DmesgResponse = rsp.data.pread_with::<DmesgResponse>(0, LE)?;

        Ok(res)
    }
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
