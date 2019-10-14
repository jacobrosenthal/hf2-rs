use crate::command::{xmit, Command, Commander, Error, NoResult};

///Reset the device into user-space app. Usually, no response at all will arrive for this command.
pub struct ResetIntoApp {}
impl Commander<NoResult> for ResetIntoApp {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let command = Command::new(0x0003, 0, vec![]);

        xmit(command, d)?;

        Ok(NoResult {})
    }
}
