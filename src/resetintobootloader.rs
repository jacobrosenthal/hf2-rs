use crate::command::{xmit, Command, Commander, Error, NoResult};


///Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
pub struct ResetIntoBootloader {}
impl Commander<NoResult> for ResetIntoBootloader {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let command = Command::new(0x0004, 0, vec![]);

        xmit(command, d)?;

        Ok(NoResult {})
    }
}
