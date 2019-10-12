///https://github.com/Microsoft/uf2/blob/master/hf2.md
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use core::convert::TryFrom;
use crc::{crc16, Hasher16};
use hidapi::HidApi;
use scroll::{ctx, Pread, Pwrite, LE};
use std::fs::File;

#[derive(Debug, Copy, Clone, PartialEq)]
enum CommandId {
    // This command states the current mode of the device:
    BinInfo = 0x0001,
    // Various device information. The result is a character array. See INFO_UF2.TXT in UF2 format for details.
    Info = 0x0002,
    //Reset the device into user-space app. Usually, no response at all will arrive for this command.
    ResetIntoApp = 0x0003,
    //Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
    ResetIntoBootloader = 0x0004,
    // When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
    StartFlash = 0x0005,
    //Write a single page of flash memory. No Result.
    WriteFlashPage = 0x0006,
    //Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
    Checksum = 0x0007,
    //Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_address must be suitably aligned. This is to support reading of special IO regions.
    ReadWords = 0x0008,
    //Dual of READ WORDS, with the same constraints. No Result.
    WriteWords = 0x0009,
    //Return internal log buffer if any. The result is a character array.
    Dmesg = 0x0010,
}

#[derive(Debug, PartialEq)]
enum BinInfoMode {
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

// This command states the current mode of the device:
struct BinInfo {}

impl Commander<BinInfoResult> for BinInfo {
    fn send(&self, d: &hidapi::HidDevice) -> Result<BinInfoResult, Error> {
        let bitsnbytes = transfer(CommandId::BinInfo, d, &[0])?;

        let res: BinInfoResult = (bitsnbytes.as_slice()).pread_with::<BinInfoResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug)]
struct BinInfoResult {
    mode: BinInfoMode, //    uint32_t mode;
    flash_page_size: u32,
    flash_num_pages: u32,
    max_message_size: u32,
    family_id: Option<u32>, // optional
}
impl CommanderResult for BinInfoResult {}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for BinInfoResult {
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

        //todo, not sure if optional means it would be 0, or would not be included at all
        let family_id = if offset < this.len() {
            Some(this.gread_with::<u32>(&mut offset, le)?)
        } else {
            None
        };

        Ok((
            BinInfoResult {
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

// Various device information. The result is a character array. See INFO_UF2.TXT in UF2 format for details.
struct Info {}
impl Commander<InfoResult> for Info {
    fn send(&self, d: &hidapi::HidDevice) -> Result<InfoResult, Error> {
        let bitsnbytes = transfer(CommandId::Info, d, &[0])?;

        let res: InfoResult = (bitsnbytes.as_slice()).pread_with::<InfoResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug)]
struct InfoResult {
    info: String,
}
impl CommanderResult for InfoResult {}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for InfoResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut bytes = vec![0; this.len()];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut bytes, le)?;

        let info = std::str::from_utf8(&bytes)?;

        Ok((InfoResult { info: info.into() }, offset))
    }
}

//Write a single page of flash memory. No Result.
struct WriteFlashPage {
    target_address: u32,
    data: Vec<u8>,
}

impl Commander<NoResult> for WriteFlashPage {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let mut data = vec![0_u8; self.data.len() + 4];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;

        for i in &self.data {
            data.gwrite_with(i, &mut offset, LE)?;
        }

        let _ = transfer(CommandId::WriteFlashPage, d, &data[..])?;

        Ok(NoResult {})
    }
}

//Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
struct ChksumPages {
    target_address: u32,
    num_pages: u32,
}

impl Commander<ChksumPagesResult> for ChksumPages {
    fn send(&self, d: &hidapi::HidDevice) -> Result<ChksumPagesResult, Error> {
        let data = &mut [0_u8; 8];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;
        data.gwrite_with(self.num_pages, &mut offset, LE)?;

        let bitsnbytes = transfer(CommandId::Checksum, d, &data[..])?;

        let res: ChksumPagesResult =
            (bitsnbytes.as_slice()).pread_with::<ChksumPagesResult>(0, LE)?;

        Ok(res)
    }
}

//Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
#[derive(Debug)]
struct ChksumPagesResult {
    chksums: Vec<u16>,
}
impl CommanderResult for ChksumPagesResult {}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ChksumPagesResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut chksums: Vec<u16> = vec![0; this.len() / 2];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut chksums, le)?;

        Ok((ChksumPagesResult { chksums }, offset))
    }
}

//Reset the device into user-space app. Usually, no response at all will arrive for this command.
struct ResetIntoApp {}
impl Commander<NoResult> for ResetIntoApp {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let _ = transfer(CommandId::ResetIntoApp, d, &[0])?;

        Ok(NoResult {})
    }
}

//Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
struct ResetIntoBootloader {}
impl Commander<NoResult> for ResetIntoBootloader {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let _ = transfer(CommandId::ResetIntoBootloader, d, &[0])?;

        Ok(NoResult {})
    }
}

// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
struct StartFlash {}
impl Commander<NoResult> for StartFlash {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let _ = transfer(CommandId::StartFlash, d, &[0])?;

        Ok(NoResult {})
    }
}

//Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
struct ReadWords {
    target_address: u32,
    num_words: u32,
}

impl Commander<ReadWordsResult> for ReadWords {
    fn send(&self, d: &hidapi::HidDevice) -> Result<ReadWordsResult, Error> {
        let data = &mut [0_u8; 8];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;
        data.gwrite_with(self.num_words, &mut offset, LE)?;

        let bitsnbytes = transfer(CommandId::ReadWords, d, &data[..])?;

        let res: ReadWordsResult = (bitsnbytes.as_slice()).pread_with::<ReadWordsResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug)]
struct ReadWordsResult {
    words: Vec<u32>,
}
impl CommanderResult for ReadWordsResult {}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for ReadWordsResult {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        let mut words: Vec<u32> = vec![0; this.len() / 4];

        let mut offset = 0;
        this.gread_inout_with(&mut offset, &mut words, le)?;

        Ok((ReadWordsResult { words }, offset))
    }
}

//Dual of READ WORDS, with the same constraints. No Result.
struct WriteWords {
    target_address: u32,
    num_words: u32,
    words: Vec<u32>,
}

impl Commander<NoResult> for WriteWords {
    fn send(&self, d: &hidapi::HidDevice) -> Result<NoResult, Error> {
        let data = &mut [0_u8; 64];

        let mut offset = 0;

        data.gwrite_with(self.target_address, &mut offset, LE)?;
        data.gwrite_with(self.num_words, &mut offset, LE)?;

        for i in &self.words {
            data.gwrite_with(i, &mut offset, LE)?;
        }

        let _ = transfer(CommandId::WriteWords, d, &data[..offset])?;

        Ok(NoResult {})
    }
}

//Return internal log buffer if any. The result is a character array.
struct Dmesg {}

impl Commander<DmesgResult> for Dmesg {
    fn send(&self, d: &hidapi::HidDevice) -> Result<DmesgResult, Error> {
        let bitsnbytes = transfer(CommandId::Dmesg, d, &[0])?;

        let res: DmesgResult = (bitsnbytes.as_slice()).pread_with::<DmesgResult>(0, LE)?;

        Ok(res)
    }
}

#[derive(Debug)]
struct DmesgResult {
    logs: String,
}
impl CommanderResult for DmesgResult {}

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

trait CommanderResult {}

trait Commander<RES: CommanderResult> {
    fn send(&self, d: &hidapi::HidDevice) -> Result<RES, Error>;
}

struct NoResult {}
impl CommanderResult for NoResult {}

#[derive(Debug)]
struct CommandResponse {
    //arbitrary number set by the host, for example as sequence number. The response should repeat the tag.
    tag: u16,
    status: CommandResponseStatus, //    uint8_t status;

    //additional information In case of non-zero status
    status_info: u8, // optional?
                     // data: Vec<u8>,
}

#[derive(Debug, PartialEq)]
enum CommandResponseStatus {
    //command understood and executed correctly
    Success = 0x00,
    //command not understood
    ParseError = 0x01,
    //command execution error
    ExecutionError = 0x02,
}

impl TryFrom<u8> for CommandResponseStatus {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(CommandResponseStatus::Success),
            1 => Ok(CommandResponseStatus::ParseError),
            2 => Ok(CommandResponseStatus::ExecutionError),
            _ => Err(Error::Parse),
        }
    }
}

#[derive(Debug, PartialEq)]
enum PacketType {
    //Inner packet of a command message
    Inner = 0,
    //Final packet of a command message
    Final = 1,
    //Serial stdout
    StdOut = 2,
    //Serial stderr
    Stderr = 3,
}

impl TryFrom<u8> for PacketType {
    type Error = Error;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0 => Ok(PacketType::Inner),
            1 => Ok(PacketType::Final),
            2 => Ok(PacketType::StdOut),
            3 => Ok(PacketType::Stderr),
            _ => Err(Error::Parse),
        }
    }
}

// doesnt know what the data is supposed to be decoded as
// thats linked via the seq number outside, so we cant decode here
impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for CommandResponse {
    type Error = Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 4 {
            return Err(Error::Parse);
        }

        let mut offset = 0;
        let tag = this.gread_with::<u16>(&mut offset, le)?;
        let status: u8 = this.gread_with::<u8>(&mut offset, le)?;
        let status = CommandResponseStatus::try_from(status)?;
        let status_info = this.gread_with::<u8>(&mut offset, le)?;

        Ok((
            CommandResponse {
                tag,
                status,
                status_info,
            },
            offset,
        ))
    }
}

fn transfer(id: CommandId, d: &hidapi::HidDevice, data: &[u8]) -> Result<Vec<u8>, Error> {
    let mut seq: u16 = 0;

    //Packets are up to 64 bytes long
    let buffer = &mut [0_u8; 64];

    //command struct is 8, header is 1
    let step = 64 - 9;
    for chunk in data.chunks(step as usize) {
        seq += 1;

        //account for header
        let mut offset = 1;

        buffer.gwrite_with(id as u32, &mut offset, LE)?;
        buffer.gwrite_with(seq, &mut offset, LE)?;
        buffer.gwrite_with(0_u8, &mut offset, LE)?;
        buffer.gwrite_with(0_u8, &mut offset, LE)?;

        // println!("{:?} {:?}", offset, data.len());

        // we dont include header in length, so offset -1
        let len: usize = offset - 1 + data.len();

        println!("{:?} {:?}", step * seq, data.len());

        let packet = if step * seq >= (data.len() as u16) {
            PacketType::Final
        } else {
            PacketType::Inner
        };

        buffer[0] = (packet as u8) << 6 | len as u8;

        let first_and_last = [&buffer[..offset], &chunk[..]].concat();

        d.write(first_and_last.as_slice())?;
    }

    //exit early for some commands
    if id == CommandId::ResetIntoApp || id == CommandId::ResetIntoBootloader {
        return Ok(vec![]);
    }

    //exit early for some commands
    if id == CommandId::ResetIntoApp || id == CommandId::ResetIntoBootloader {
        return Ok(vec![]);
    }

    let mut bitsnbytes: Vec<u8> = vec![];

    // keep reading until Final packet
    while {
        d.read(buffer)?;

        let ptype = PacketType::try_from(buffer[0] >> 6)?;
        let len: usize = (buffer[0] & 0x3F) as usize;
        println!(
            "header: {:02X?} (ptype: {:?} len: {:?}) data: {:02X?}",
            &buffer[0],
            ptype,
            len,
            &buffer[1..=len]
        );

        //skip the header byte and strip excess bytes remote is allowed to send
        bitsnbytes.extend_from_slice(&buffer[1..=len]);

        ptype == PacketType::Inner
    } {}

    let mut offset = 0;

    let resp = bitsnbytes
        .as_slice()
        .gread_with::<CommandResponse>(&mut offset, LE)?;

    if resp.status != CommandResponseStatus::Success {
        return Err(Error::MalformedRequest);
    }

    Ok(bitsnbytes[offset..].to_vec())
}

#[derive(Clone, Debug)]
pub(crate) enum Error {
    Arguments,
    Parse,
    MalformedRequest,
    Execution,
    Sequence,
    Transmission,
}

impl From<hidapi::HidError> for Error {
    fn from(_err: hidapi::HidError) -> Self {
        Error::Transmission
    }
}

impl From<scroll::Error> for Error {
    fn from(_err: scroll::Error) -> Self {
        Error::Parse
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(_err: std::str::Utf8Error) -> Self {
        Error::Parse
    }
}

impl From<std::io::Error> for Error {
    fn from(_err: std::io::Error) -> Self {
        Error::Arguments
    }
}

fn main() -> Result<(), Error> {
    let api = HidApi::new().expect("Couldn't find system usb");

    let d = api.open(0x239A, 0x003D).expect("Couldn't find usb device");

    let bininfo: BinInfoResult = BinInfo {}.send(&d)?;
    println!(
        "{:?} {:?}kb",
        bininfo,
        bininfo.flash_num_pages * bininfo.flash_page_size / 1024
    );

    let info: InfoResult = Info {}.send(&d)?;
    println!("{:?}", info);

    // not supported on my board
    // let dmesg: DmesgResult = Dmesg {}.send(&d)?;
    // println!("{:?}", dmesg);

    if bininfo.mode != BinInfoMode::Bootloader {
        let _ = StartFlash {}.send(&d)?;
    }

    let starting_address = 0x4000; //todo samd51, get out of the binary or from an arg
    let top_address = bininfo.flash_num_pages * bininfo.flash_page_size;

    {
        //upload

        let mut f = File::open("./ferris_img.bin")?;
        let mut binary = Vec::new();
        //shouldnt there be a chunking interator for htis?
        f.read_to_end(&mut binary)?;

        use std::io::prelude::*;

        //sigh, cant enumerate, right?
        let mut page_index = 0;
        for data in binary.chunks(bininfo.flash_page_size as usize) {
            let target_address = starting_address + bininfo.flash_page_size * page_index;
            // println!("{:02X?}", target_address);
            // println!("{:?}", data.len());
            // println!("{:02X?}", data);
            let _ = WriteFlashPage {
                target_address,
                data: data.to_vec(),
            }
            .send(&d)?;

            // println!("{:?}", crc16::checksum_x25(data));

            page_index += 1;
        }
    }

    // println!("done");
    // {
    //     //checksums
    //     let max_pages = bininfo.max_message_size / 2 - 2;
    //     let steps = max_pages * bininfo.flash_page_size;

    //     for target_address in (starting_address..top_address).step_by(steps as usize) {
    //         let pages_left = (top_address - target_address) / bininfo.flash_page_size;

    //         let num_pages = if pages_left < max_pages {
    //             pages_left
    //         } else {
    //             max_pages
    //         };
    //         let chk: ChksumPagesResult = ChksumPages {
    //             target_address,
    //             num_pages,
    //         }
    //         .send(&d)?;
    //         println!("{:?}", chk);
    //     }
    // }
    let _ = ResetIntoApp {}.send(&d)?;

    // {
    //     //read words
    //     //max? just use flash page size for now

    //     let mut device_binary = vec![];
    //     let bytes = bininfo.flash_page_size; //256
    //     for target_address in (starting_address..top_address).step_by(bytes as usize) {
    //         let bytes_left = top_address - target_address;

    //         let num_words = if bytes_left < bytes {
    //             bytes_left / 32
    //         } else {
    //             bytes / 32
    //         };
    //         println!("{:?}", bytes_left);
    //         println!("{:02X?} {:?}", target_address, num_words);

    //         let res: ReadWordsResult = ReadWords {
    //             target_address,
    //             num_words,
    //         }
    //         .send(&d)?;

    //         device_binary.extend_from_slice(&res.words[..]);
    //     }
    // }

    // let _ = WriteWords {
    //     target_addr: 0,
    //     num_words: 1,
    //     words: vec![],
    // .send(&d)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packetize() {
        let message = vec![
            0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0xD0, 0x09, 0x0A, 0x0B, 0x0C, 0x0D,
            0x0E, 0x0F, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17,
        ];
        // Packet 0: 83 01 02 03 AB FF FF FF
        // Packet 1: 85 04 05 06 07 08
        // Packet 2: 80 DE 42 42 42 42 FF FF
        // Packet 3: D0 09 0A 0B 0C 0D 0E 0F 10 11 12 13 14 15 16 17 FF FF FF

        unimplemented!();
    }
}
