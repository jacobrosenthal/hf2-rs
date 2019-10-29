use crate::command::Response;
use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use core::convert::TryFrom;

/// Return internal log buffer if any.
pub struct Dmesg {}

impl<'a> Commander<'a> for Dmesg {
    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        xmit(0x0010, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        Ok(Response::Dmesg(DmesgResponse::try_from(rsp.data)?))
    }
}

#[derive(Debug, PartialEq)]
pub struct DmesgResponse<'a> {
    pub logs: &'a str,
}

impl<'a> TryFrom<&'a [u8]> for DmesgResponse<'a> {
    type Error = Error;

    fn try_from(this: &'a [u8]) -> Result<DmesgResponse<'a>, Self::Error> {
        //u8, no endianness
        let logs = core::str::from_utf8(&this)?;

        Ok(DmesgResponse { logs })
    }
}
