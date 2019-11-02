use crate::command::{xmit, Commander, Error, NoResponse};
use crate::mock::HidMockable;

/// Reset the device into user-space app.
pub struct ResetIntoApp {}

impl<'a> Commander<'a, NoResponse> for ResetIntoApp {
    const ID: u32 = 0x0003;

    fn send<T: HidMockable>(&self, _data: &'a mut [u8], d: &T) -> Result<NoResponse, Error> {
        xmit(Self::ID, 0, &[], d)?;

        Ok(NoResponse {})
    }
}
