use crate::command::{rx, xmit, Commander, Error, NoResponse};
use crate::mock::HidMockable;

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
pub struct StartFlash {}

impl<'a> Commander<'a, NoResponse> for StartFlash {
    const ID: u32 = 0x0005;

    fn send<T: HidMockable>(&self, data: &'a mut [u8], d: &T) -> Result<NoResponse, Error> {
        xmit(Self::ID, 0, &[], d)?;

        let _ = rx(data, d)?;

        Ok(NoResponse {})
    }
}
