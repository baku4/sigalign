#[cfg(target_endian = "little")]
pub type EndianType = byteorder::LittleEndian;
#[cfg(target_endian = "big")]
pub type EndianType = byteorder::BigEndian;
pub use byteorder::{ReadBytesExt, WriteBytesExt};