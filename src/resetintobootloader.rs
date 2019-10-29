use crate::command::Response;
use crate::command::{xmit, Commander, Error};

/// Reset the device into bootloader, usually for flashing.
pub struct ResetIntoBootloader {}

impl<'a> Commander<'a> for ResetIntoBootloader {
    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        xmit(0x0004, 0, &data, d)?;

        Ok(Response::NoResponse)
    }
}
