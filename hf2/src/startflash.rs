use crate::command::{rx, xmit, Command};
use crate::Error;

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that. Empty tuple response.
pub fn start_flash(d: &hidapi::HidDevice) -> Result<(), Error> {
    xmit(Command::new(0x0005, 0, vec![]), d)?;

    rx(d).map(|_| ())
}
