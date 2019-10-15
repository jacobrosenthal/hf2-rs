use crate::command::{rx, xmit, Command, CommandResponseStatus, Commander, Error};
use scroll::{ctx, Pread, LE};

///Return internal log buffer if any. The result is a character array.
pub struct Dmesg {}

impl<'a> Commander<'a, DmesgResult> for Dmesg {
    fn send(&self, d: &hidapi::HidDevice) -> Result<DmesgResult, Error> {
        let command = Command::new(0x0010, 0, vec![]);

        xmit(command, d)?;

        let rsp = rx(d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::MalformedRequest);
        }

        let res: DmesgResult = (rsp.data.as_slice()).pread_with::<DmesgResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub struct DmesgResult {
    pub logs: String,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for DmesgResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut bytes = vec![0; this.len()];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut bytes, le)?;

        let logs = std::str::from_utf8(&bytes)?;

        Ok((DmesgResult { logs: logs.into() }, offset))
    }
}
