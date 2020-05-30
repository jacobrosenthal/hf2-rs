/// This command states the current mode of the device:
mod bininfo;
pub use bininfo::*;

///Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
mod checksumpages;
pub use checksumpages::*;

///Return internal log buffer if any. The result is a character array.
mod dmesg;
pub use dmesg::*;

/// Various device information. The result is a character array. See INFO_UF2.TXT in UF2 format for details.
mod info;
pub use info::*;

///Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
mod readwords;
pub use readwords::*;

///Reset the device into user-space app. Usually, no response at all will arrive for this command.
mod resetintoapp;
pub use resetintoapp::*;

///Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
mod resetintobootloader;
pub use resetintobootloader::*;

/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
mod startflash;
pub use startflash::*;

///Write a single page of flash memory. No Result.
mod writeflashpage;
pub use writeflashpage::*;

///Dual of READ WORDS, with the same constraints. No Result.
mod writewords;
pub use writewords::*;

/// Errors and traits to build a command
mod command;

#[derive(Clone, Debug)]
pub enum Error {
    Arguments,
    Parse,
    CommandNotRecognized,
    Execution,
    Sequence,
    Transmission,
}

///trait to implement HID devices
pub trait ReadWrite {
    fn hf2_write(&self, data: &[u8]) -> Result<usize, Error>;
    fn hf2_read(&self, buf: &mut [u8]) -> Result<usize, Error>;
}

#[cfg(feature = "hidapi")]
mod hidapi_trait;
