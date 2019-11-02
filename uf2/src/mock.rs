use crate::command::*;

#[allow(dead_code)]
pub struct MyMock<'a, R, W>
where
    R: Fn() -> &'a [u8],
    W: Fn(&[u8]) -> usize,
{
    pub reader: R,
    pub writer: W,
}

pub trait HidMockable {
    fn my_write(&self, data: &[u8]) -> Result<usize, Error>;
    fn my_read(&self, buf: &mut [u8]) -> Result<usize, Error>;
}

impl<'a, R, W> HidMockable for MyMock<'a, R, W>
where
    R: Fn() -> &'a [u8],
    W: Fn(&[u8]) -> usize,
{
    fn my_write(&self, data: &[u8]) -> Result<usize, Error> {
        let len = (&self.writer)(data);

        Ok(len)
    }
    fn my_read(&self, buf: &mut [u8]) -> Result<usize, Error> {
        let data = (self.reader)();

        for (i, val) in data.iter().enumerate() {
            buf[i] = *val
        }

        Ok(data.len())
    }
}
