use crate::command::Response;
use crate::command::{rx, xmit, Commander, Error};

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
pub struct StartFlash {}

impl<'a> Commander<'a> for StartFlash {
    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        xmit(0x0005, 0, &data, d)?;

        let _ = rx(data, d)?;

        Ok(Response::NoResponse)
    }
}
