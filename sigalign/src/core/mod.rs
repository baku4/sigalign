pub mod regulators;

mod sequence_length;
pub use sequence_length::SeqLen;

// Reference
pub trait ReferenceInterface {
    type Buffer: SequenceBuffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation>;
    fn get_buffer(&self) -> Self::Buffer;
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer);
    fn is_valid(&self, query: &[u8]) -> bool;
}
pub trait SequenceBuffer {
    fn request_sequence(&self) -> &[u8];
}

/// The index of pattern.
///
/// Positions are should be sorted in ascending order.
///   - In general, positions are automatically sorted when searching for an index of a target.
///   - Reordering is not performed in algorithm.
/// The range of position in one target is restricted to the bound of `u32`
#[derive(Debug)]
pub struct PatternLocation {
    pub target_index: u32,
    pub sorted_positions: Vec<u32>,
}

// Extension for serialization
#[cfg(target_endian = "little")]
pub(crate) type EndianType = byteorder::LittleEndian;
#[cfg(target_endian = "big")]
pub(crate) type EndianType = byteorder::BigEndian;
pub(crate) use byteorder::{ReadBytesExt, WriteBytesExt};
