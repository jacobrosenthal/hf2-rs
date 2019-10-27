use crate::command::{xmit, Command, Commander, Error, NoResponse};

///Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
pub struct ResetIntoBootloader {}

impl<'a> Commander<'a, NoResponse> for ResetIntoBootloader {
    const ID: u32 = 0x0004;

    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        let command = Command::new(Self::ID, 0, vec![]);

        xmit(command, d)?;

        Ok(NoResponse {})
    }
}
