use crate::command::{rx, xmit, Command, CommandResponse, CommandResponseStatus};
use crate::Error;
use scroll::{ctx, Pread, Pwrite, LE};

///Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
pub fn checksum_pages(
    d: &hidapi::HidDevice,
    target_address: u32,
    num_pages: u32,
) -> Result<ChecksumPagesResponse, Error> {
    let mut buffer = vec![0_u8; 8];
    let mut offset = 0;

    buffer.gwrite_with(target_address, &mut offset, scroll::LE)?;
    buffer.gwrite_with(num_pages, &mut offset, scroll::LE)?;

    xmit(Command::new(0x0007, 0, buffer), d)?;

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

///Response to the checksum_pages command
#[derive(Debug, PartialEq)]
pub struct ChecksumPagesResponse {
    pub checksums: Vec<u16>,
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ChecksumPagesResponse {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 2 {
            return Err(Error::Parse);
        }

        let mut checksums: Vec<u16> = vec![0; this.len() / 2];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut checksums, le)?;

        Ok((ChecksumPagesResponse { checksums }, offset))
    }
}
