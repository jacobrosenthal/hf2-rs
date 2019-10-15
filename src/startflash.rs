use crate::command::{rx, xmit, Command, Commander, Error, NoResult};

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
pub struct StartFlash {}

impl<'a> Commander<'a, NoResult> for StartFlash {
    const ID: u32 = 0x0005;

    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let command = Command::new(Self::ID, 0, vec![]);

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResult {})
    }
}
