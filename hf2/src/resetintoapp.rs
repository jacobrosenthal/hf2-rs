use crate::command::{xmit, Command, Commander, NoResponse};
use crate::Error;

///Reset the device into user-space app. Usually, no response at all will arrive for this command.
pub struct ResetIntoApp {}

impl<'a> Commander<'a, NoResponse> for ResetIntoApp {
    const ID: u32 = 0x0003;

    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResponse, Error> {
        let command = Command::new(Self::ID, 0, vec![]);

        xmit(command, d)?;

        Ok(NoResponse {})
    }
}
