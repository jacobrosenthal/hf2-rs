use crate::command::{xmit, Command, Commander, Error, NoResult};

///Reset the device into user-space app. Usually, no response at all will arrive for this command.
pub struct ResetIntoApp {}

impl<'a> Commander<'a, NoResult> for ResetIntoApp {
    const ID: u32 = 0x0003;

    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let command = Command::new(Self::ID, 0, vec![]);

        xmit(command, d)?;

        Ok(NoResult {})
    }
}
