use crate::command::{rx, xmit, Command, CommandResponse, CommandResponseStatus};
use crate::Error;
use scroll::{ctx, Pread, LE};

///Return internal log buffer if any. The result is a character array.

pub fn dmesg(d: &hidapi::HidDevice) -> Result<DmesgResponse, Error> {
    xmit(Command::new(0x0010, 0, vec![]), d)?;

    match rx(d) {
        Ok(CommandResponse {
            status: CommandResponseStatus::Success,
            data,
            ..
        }) => (data.as_slice()).pread_with(0, LE),
        Ok(_) => Err(Error::CommandNotRecognized),
        Err(e) => Err(e),
    }
}

///Response to the dmesg command
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
