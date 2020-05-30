use crate::command::{rx, xmit, Command, CommandResponseStatus, Commander};
use crate::Error;
use scroll::{ctx, Pread, LE};

///Return internal log buffer if any. The result is a character array.
pub struct Dmesg {}

impl<'a> Commander<'a, DmesgResponse> for Dmesg {
    const ID: u32 = 0x0010;

    fn send(&self, d: &hidapi::HidDevice) -> Result<DmesgResponse, Error> {
        let command = Command::new(Self::ID, 0, vec![]);

        xmit(command, d)?;

        let rsp = rx(d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        let res: DmesgResponse = (rsp.data.as_slice()).pread_with::<DmesgResponse>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug, PartialEq)]
pub struct DmesgResponse {
    pub logs: String,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for DmesgResponse {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut bytes = vec![0; this.len()];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut bytes, le)?;

        let logs = core::str::from_utf8(&bytes)?;

        Ok((DmesgResponse { logs: logs.into() }, offset))
    }
}
