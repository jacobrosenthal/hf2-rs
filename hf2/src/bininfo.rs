use crate::command::{rx, xmit, Command, CommandResponse, CommandResponseStatus};
use crate::Error;
use core::convert::TryFrom;
use scroll::{ctx, Pread, LE};

#[derive(Debug, PartialEq)]
pub enum BinInfoMode {
    //bootloader, and thus flashing of user-space programs is allowed
    Bootloader = 0x0001,
    //user-space mode. It also returns the size of flash page size (flashing needs to be done on page-by-page basis), and the maximum size of message. It is always the case that max_message_size >= flash_page_size + 64.
    User = 0x0002,
}

impl TryFrom<u32> for BinInfoMode {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(BinInfoMode::Bootloader),
            2 => Ok(BinInfoMode::User),
            _ => Err(Error::Parse),
        }
    }
}

/// This command states the current mode of the device:
pub fn bin_info(d: &hidapi::HidDevice) -> Result<BinInfoResponse, Error> {
    xmit(Command::new(0x0001, 0, vec![]), d)?;

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

///Response to the bin_info command
#[derive(Debug, PartialEq)]
pub struct BinInfoResponse {
    pub mode: BinInfoMode, //    uint32_t mode;
    pub flash_page_size: u32,
    pub flash_num_pages: u32,
    pub max_message_size: u32,
    pub family_id: Option<FamilyId>,
}

#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum FamilyId {
    ATSAMD21,
    ATSAMD51,
    NRF52840,
    STM32F103,
    STM32F401,
    ATMEGA32,
    CYPRESS_FX2,
    UNKNOWN(u32),
}

impl From<u32> for FamilyId {
    fn from(val: u32) -> Self {
        match val {
            0x68ed_2b88 => Self::ATSAMD21,
            0x5511_4460 => Self::ATSAMD51,
            0x1b57_745f => Self::NRF52840,
            0x5ee2_1072 => Self::STM32F103,
            0x5775_5a57 => Self::STM32F401,
            0x1657_3617 => Self::ATMEGA32,
            0x5a18_069b => Self::CYPRESS_FX2,
            _ => Self::UNKNOWN(val),
        }
    }
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for BinInfoResponse {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 16 {
            return Err(Error::Parse);
        }

        //does it give me offset somehow??? or just slice appropriately for me?s
        let mut offset = 0;
        let mode: u32 = this.gread_with::<u32>(&mut offset, le)?;
        let mode: Result<BinInfoMode, Error> = BinInfoMode::try_from(mode);
        let mode: BinInfoMode = mode?;
        let flash_page_size = this.gread_with::<u32>(&mut offset, le)?;
        let flash_num_pages = this.gread_with::<u32>(&mut offset, le)?;
        let max_message_size = this.gread_with::<u32>(&mut offset, le)?;

        let family_id = if this.len() >= 20 {
            let family_id: FamilyId = this.gread_with::<u32>(&mut offset, le)?.into();
            Some(family_id)
        } else {
            None
        };

        Ok((
            BinInfoResponse {
                mode,
                flash_page_size,
                flash_num_pages,
                max_message_size,
                family_id,
            },
            offset,
        ))
    }
}
