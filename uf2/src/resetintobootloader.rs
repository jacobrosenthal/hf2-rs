use crate::command::{xmit, Commander, Error, NoResponse};
use crate::mock::HidMockable;

/// Reset the device into bootloader, usually for flashing.
pub struct ResetIntoBootloader {}

impl<'a> Commander<'a, NoResponse> for ResetIntoBootloader {
    const ID: u32 = 0x0004;

    fn send<T: HidMockable>(&self, _data: &'a mut [u8], d: &T) -> Result<NoResponse, Error> {
        xmit(Self::ID, 0, &[], d)?;

        Ok(NoResponse {})
    }
}
