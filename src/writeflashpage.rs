use crate::command::{rx, xmit, Command, Commander, Error, NoResult};
use scroll::{Pwrite, LE};

///Write a single page of flash memory. No Result.
pub struct WriteFlashPage {
    pub target_address: u32,
    pub data: Vec<u8>,
}

impl Commander<NoResult> for WriteFlashPage {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let mut data = vec![0_u8; self.data.len() + 4];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;

        for i in &self.data {
            data.gwrite_with(i, &mut offset, LE)?;
        }

        let command = Command::new(0x0006, 0, data);

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResult {})
    }
}
