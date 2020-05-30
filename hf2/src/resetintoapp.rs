use crate::command::{xmit, Command};
use crate::Error;

///Reset the device into user-space app. Empty tuple response.
pub fn reset_into_app(d: &hidapi::HidDevice) -> Result<(), Error> {
    xmit(Command::new(0x0003, 0, vec![]), d)
}
