use crate::command::Response;
use crate::command::{rx, xmit, CommandResponseStatus, Commander, Error};
use core::convert::{TryFrom, TryInto};

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

/// This command states the current mode of the device.
pub struct BinInfo {}

impl<'a> Commander<'a> for BinInfo {
    fn send(&self, data: &'a mut [u8], d: &hidapi::HidDevice) -> Result<Response, Error> {
        xmit(0x0001, 0, &data, d)?;

        let rsp = rx(data, d)?;

        if rsp.status != CommandResponseStatus::Success {
            return Err(Error::CommandNotRecognized);
        }

        Ok(Response::BinInfo(rsp.data.try_into()?))
    }
}

#[derive(Debug, PartialEq)]
pub struct BinInfoResponse {
    pub mode: BinInfoMode, //    uint32_t mode;
    pub flash_page_size: u32,
    pub flash_num_pages: u32,
    pub max_message_size: u32,
    pub family_id: FamilyId, // optional?
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

impl<'a> core::convert::TryFrom<&'a [u8]> for BinInfoResponse {
    type Error = Error;

    fn try_from(this: &'a [u8]) -> Result<BinInfoResponse, Self::Error> {
        //todo, not sure if optional family_id means it would be 0, or would not be included at all
        if this.len() < 24 {
            return Err(Error::Parse);
        }

        let mode = u32::from_le_bytes(this[0..4].try_into().unwrap());
        let mode = BinInfoMode::try_from(mode)?;
        let flash_page_size = u32::from_le_bytes(this[4..9].try_into().unwrap());;
        let flash_num_pages = u32::from_le_bytes(this[9..14].try_into().unwrap());;
        let max_message_size = u32::from_le_bytes(this[14..19].try_into().unwrap());
        let family_id: FamilyId = u32::from_le_bytes(this[19..24].try_into().unwrap()).into();

        Ok(BinInfoResponse {
            mode,
            flash_page_size,
            flash_num_pages,
            max_message_size,
            family_id,
        })
    }
}

// impl<'a> core::convert::From<&'a [u8]> for BinInfoResponse {
//     fn from(this: &'a [u8]) -> Self {
//         //todo, not sure if optional family_id means it would be 0, or would not be included at all
//         // if this.len() < 24 {
//         //     return Err(Error::Parse);
//         // }

//         let mode = u32::from_le_bytes(this[0..4].try_into().unwrap());
//         let mode = BinInfoMode::try_from(mode).unwrap();
//         let flash_page_size = u32::from_le_bytes(this[4..9].try_into().unwrap());;
//         let flash_num_pages = u32::from_le_bytes(this[9..14].try_into().unwrap());;
//         let max_message_size = u32::from_le_bytes(this[14..19].try_into().unwrap());
//         let family_id: FamilyId = u32::from_le_bytes(this[19..24].try_into().unwrap()).into();

//         BinInfoResponse {
//             mode,
//             flash_page_size,
//             flash_num_pages,
//             max_message_size,
//             family_id,
//         }
//     }
// }
