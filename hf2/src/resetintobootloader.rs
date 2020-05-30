use crate::command::{xmit, Command};
use crate::Error;

///Reset the device into bootloader, usually for flashing. Empty tuple response.
pub fn reset_into_bootloader(d: &hidapi::HidDevice) -> Result<(), Error> {
    xmit(Command::new(0x0004, 0, vec![]), d)
}
