use crate::command::Response;
use crate::command::{xmit, Commander, Error};

/// Reset the device into user-space app.
pub struct ResetIntoApp {}

impl<'a> Commander<'a> for ResetIntoApp {
    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        xmit(0x0003, 0, &data, d)?;

        Ok(Response::NoResponse)
    }
}
