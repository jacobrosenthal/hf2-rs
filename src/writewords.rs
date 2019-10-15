use crate::command::{rx, xmit, Command, Commander, Error, NoResult};
use scroll::{Pwrite, LE};

///Dual of READ WORDS, with the same constraints. No Result.
pub struct WriteWords {
    pub target_address: u32,
    pub num_words: u32,
    pub words: Vec<u32>,
}

impl<'a> Commander<'a, NoResult> for WriteWords {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let data = &mut [0_u8; 64];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;
        data.gwrite_with(self.num_words, &mut offset, LE)?;

        for i in &self.words {
            data.gwrite_with(i, &mut offset, LE)?;
        }

        let command = Command::new(0x0009, 0, data.to_vec());

        xmit(command, d)?;

        let _ = rx(d)?;

        Ok(NoResult {})
    }
}
