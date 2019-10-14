use hidapi::HidDevice;
use hidapi::HidResult;

#[allow(dead_code)]
pub struct MyMock<R, W>
where
    R: Fn() -> Vec<u8>,
    W: Fn(&[u8]) -> usize,
{
    pub reader: R,
    pub writer: W,
}

pub trait HidMockable {
    fn my_write(&self, data: &[u8]) -> HidResult<usize>;
    fn my_read(&self, buf: &mut [u8]) -> HidResult<usize>;
}

impl HidMockable for HidDevice {
    fn my_write(&self, data: &[u8]) -> HidResult<usize> {
        self.write(data)
    }
    fn my_read(&self, buf: &mut [u8]) -> HidResult<usize> {
        self.read(buf)
    }
}

impl<R, W> HidMockable for MyMock<R, W>
where
    R: Fn() -> Vec<u8>,
    W: Fn(&[u8]) -> usize,
{
    fn my_write(&self, data: &[u8]) -> HidResult<usize> {
        let len = (&self.writer)(data);

        Ok(len)
    }
    fn my_read(&self, buf: &mut [u8]) -> HidResult<usize> {
        let data = (self.reader)();

        for (i, val) in data.iter().enumerate() {
            buf[i] = *val
        }

        Ok(data.len())
    }
}
