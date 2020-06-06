use crate::{Error, ReadWrite};
use hidapi::HidDevice;

impl ReadWrite for HidDevice {
    fn hf2_write(&self, data: &[u8]) -> Result<usize, Error> {
        self.write(data).map_err(|e| e.into())
    }
    fn hf2_read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        self.read_timeout(buf, 1000).map_err(|e| e.into())
    }
}

impl From<hidapi::HidError> for Error {
    fn from(_err: hidapi::HidError) -> Self {
        Error::Transmission
    }
}
