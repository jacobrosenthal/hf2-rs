use crate::command::{xmit, Commander, Error, NoResponse};

/// Reset the device into user-space app.
pub struct ResetIntoApp {}

impl<'a> Commander<'a, NoResponse> for ResetIntoApp {
    const ID: u32 = 0x0003;

    fn send(&self, _data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        xmit(Self::ID, 0, &[], d)?;

        Ok(NoResponse {})
    }
}
