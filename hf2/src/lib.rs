/// Errors and traits to build a command
pub mod command;
pub use command::*;
///trait to implement HID devices
pub mod mock;
pub use mock::*;
/// This command states the current mode of the device:
pub mod bininfo;
pub use bininfo::*;
///Compute checksum of a number of pages. Maximum value for num_pages is max_message_size / 2 - 2. The checksum algorithm used is CRC-16-CCITT.
pub mod chksumpages;
pub use chksumpages::*;
///Return internal log buffer if any. The result is a character array.
pub mod dmesg;
pub use dmesg::*;
/// Various device information. The result is a character array. See INFO_UF2.TXT in UF2 format for details.
pub mod info;
pub use info::*;
///Read a number of words from memory. Memory is read word by word (and not byte by byte), and target_addr must be suitably aligned. This is to support reading of special IO regions.
pub mod readwords;
pub use readwords::*;
///Reset the device into user-space app. Usually, no response at all will arrive for this command.
pub mod resetintoapp;
pub use resetintoapp::*;
///Reset the device into bootloader, usually for flashing. Usually, no response at all will arrive for this command.
pub mod resetintobootloader;
pub use resetintobootloader::*;
/// When issued in bootloader mode, it has no effect. In user-space mode it causes handover to bootloader. A BININFO command can be issued to verify that.
pub mod startflash;
pub use startflash::*;
///Write a single page of flash memory. No Result.
pub mod writeflashpage;
pub use writeflashpage::*;
///Dual of READ WORDS, with the same constraints. No Result.
pub mod writewords;
pub use writewords::*;
