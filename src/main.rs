///https://github.com/Microsoft/uf2/blob/master/hf2.md
use byteorder::{LittleEndian, WriteBytesExt};
use hidapi::HidApi;
use scroll::{ctx, Cread, Cwrite, Pread, Pwrite, LE};

#[macro_use]
extern crate scroll;

#[derive(Debug, Copy, Clone)]
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
    //Write a single page of flash memory.
    WriteFlashPage = 0x0006,
    //Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
    Checksum = 0x0007,
    //Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
    ReadWords = 0x0008,
    //Dual of READ WORDS, with the same constraints.
    WriteWords = 0x0009,
    //Return internal log buffer if any. The result is a character array.
    Dmesg = 0x0010,
}

enum BinInfoMode {
    //bootloader, and thus flashing of user-space programs is allowed
    Bootloader = 0x01,
    //user-space mode. It also returns the size of flash page size (flashing needs to be done on page-by-page basis), and the maximum size of message. It is always the case that max_message_size >= flash_page_size + 64.
    User = 0x02,
}

struct BinInfoResult {
    mode: BinInfoMode,
    flash_page_size: u32,
    flash_num_pages: u32,
    max_message_size: u32,
    family_id: u32, // optional
}

struct InfoResult {
    info: String,
}

struct WriteFlashPageCommand {
    target_addr: u32,
    data: Vec<u8>,
}

struct ChksumPagesCommand {
    target_addr: u32,
    num_pages: u32,
}

//Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
struct ChksumPagesResult {
    chksums: Vec<u16>,
}

struct ReadWordsCommand {
    target_addr: u32,
    num_words: u32,
}
struct ReadWordsResult {
    words: Vec<u32>,
}

struct WriteWordsCommand {
    target_addr: u32,
    num_words: u32,
    words: Vec<u32>,
}

// no arguments
struct DmesgResult {
    logs: String,
}

struct Command {
    command_id: CommandId, //    uint32_t command_id;
    //arbitrary number set by the host, for example as sequence number. The response should repeat the tag.
    tag: u16,
    //The two reserved bytes in the command should be sent as zero and ignored by the device
    _reserved0: u8,
    _reserved1: u8,
    data: Vec<u8>,
}

#[derive(Debug)]
struct CommandResponse {
    //arbitrary number set by the host, for example as sequence number. The response should repeat the tag.
    tag: u16,
    status: CommandResponseStatus, //    uint8_t status;

    //additional information In case of non-zero status
    status_info: u8,
    data: Vec<u8>,
}

impl<'a> ::scroll::ctx::TryIntoCtx<::scroll::Endian> for &'a Command {
    type Error = ::scroll::Error;

    fn try_into_ctx(
        self,
        dst: &mut [u8],
        ctx: ::scroll::Endian,
    ) -> ::scroll::export::result::Result<usize, Self::Error> {
        let mut offset = 0;
        dst.gwrite_with(self.command_id as u32, &mut offset, ctx)?;
        dst.gwrite_with(&self.tag, &mut offset, ctx)?;
        dst.gwrite_with(&self._reserved0, &mut offset, ctx)?;
        dst.gwrite_with(&self._reserved1, &mut offset, ctx)?;
        // dst.gwrite_with(&self.data, offset, ctx)?;
        Ok(offset)
    }
}

#[derive(Debug)]
enum CommandResponseStatus {
    //command understood and executed correctly
    Success = 0x00,
    //command not understood
    Wut = 0x01,
    //command execution error
    Error = 0x02,
}

impl From<u8> for CommandResponseStatus {
    fn from(val: u8) -> Self {
        match val {
            0 => CommandResponseStatus::Success,
            0 => CommandResponseStatus::Wut,
            0 => CommandResponseStatus::Error,
            _ => unreachable!(),
        }
    }
}

impl<'a> ctx::TryFromCtx<'a, scroll::Endian> for CommandResponse {
    type Error = scroll::Error;
    fn try_from_ctx(this: &'a [u8], le: scroll::Endian) -> Result<(Self, usize), Self::Error> {
        if this.len() < 8 {
            return Err((scroll::Error::Custom("whatever".to_string())).into());
        }

        let mut offset = 0;
        let tag = this.gread_with::<u16>(&mut offset, le)?;
        let status: CommandResponseStatus = this.gread_with::<u8>(&mut offset, le)?.into();
        let status_info = this.gread_with::<u8>(&mut offset, le)?;
        let mut data: Vec<u8> = vec![];
        this.gread_inout_with(&mut offset, &mut data, le)?;

        Ok((
            CommandResponse {
                tag,
                status,
                status_info,
                data,
            },
            offset,
        ))
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Error {
    NotEnoughSpace,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = HidApi::new().expect("Failed to create API instance");

    let d = api.open(0x239A, 0x003D).expect("Failed to open device");

    //All words in HF2 are little endian.

    // for n bytes of data create n packets

    // let max_message_size = 320;
    // for (i, data) in buffer.chunks(max_message_size).enumerate() {
    //     // the type of the packet, in the two high bits. 0x00 inner 0x01 final
    //     // length of the remaining data (payload) in the packet, in the lower 6 bits, i.e., between 0 and 63 inclusive
    //     let header: u8 = if last {
    //         0x1 << 6 & data.len() as u8
    //     } else {
    //         data.len() as u8
    //     };
    // }
    // let serialized: Vec<u8> = serialize(&command, Infinite).unwrap();
    // println!("serialized = {:?}", serialized);
    let mut seq: u16 = 0;

    // if let Ok(context) = rusb::Context::new() {
    //     if let Ok(devices) = context.devices() {
    //         for d in devices.iter() {

    //             println!("{:?}", d);

    let command = Command {
        command_id: CommandId::BinInfo,
        //arbitrary number set by the host, for example as sequence number. The response should repeat the tag.
        tag: seq,
        //The two reserved bytes in the command should be sent as zero and ignored by the device
        _reserved0: 0,
        _reserved1: 0,
        data: vec![],
    };

    // Write it back to a buffer
    let buffer = &mut [0; 24];

    let bytes = buffer.pwrite_with(&command, 0, LE).unwrap();

    // let idx = command.pwrite_with::<Command>(&buffer, 0)?;

    // command.try_into_ctx(&buffer, 0, LE)?;
    println!("serialized buffer: {:02X?}", &buffer);

    // let response = send_command(d, command);

    // command.to_bytes(buffer).unwrap();
    d.write(&buffer[0..bytes]).unwrap();

    // Read a single `Data` at offset zero in big-endian byte order.

    d.read(buffer).unwrap();
    println!("Receive buffer: {:02X?}", &buffer[..]);

    let resp = buffer.pread_with::<CommandResponse>(0, LE)?;

    println!("deser rsp: {:?}", resp);

    // cursor.write_u32::<LittleEndian>(command.command_id as u32)?;
    // cursor.write_u16::<LittleEndian>(command.tag)?;
    // cursor.write_u8(command.reserved0)?;
    // cursor.write_u8(command.reserved1)?;
    // cursor.write_u32::<LittleEndian>(0xffff)?;

    // println!("Send buffer: {:02X?}", &buffer[..]);

    // Read back resonse.
    // TODO: Error handling & real USB reading.
    // let buffer = &mut [0; 4];
    // d.read(buffer).unwrap();
    // println!("Receive buffer: {:02X?}", &buffer[..]);
    // let response = CommandResponse::from_bytes(buffer);

    // println!("{:?}", response);
    // seq += 1;

    //         }
    //     }
    // }

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
