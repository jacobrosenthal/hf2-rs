use crate::command::{xmit, Commander, Error, NoResponse};

///Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
pub struct ResetIntoBootloader {}

impl<'a> Commander<'a, NoResponse> for ResetIntoBootloader {
    const ID: u32 = 0x0004;

    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        xmit(Self::ID, 0, &data, d)?;

        Ok(NoResponse {})
    }
}
