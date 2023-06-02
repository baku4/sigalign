pub mod regulators;

mod sequence_length;
pub use sequence_length::SeqLen;

/// `BufferedPatternSearch` represents types that can perform pattern searches within a buffered sequence.
///
/// This trait serves two main purposes:
///   - It provides pattern locations for algorithms.
///   - It acts as an interface between `Aligner` and `Reference` structs.
pub trait BufferedPatternSearch {
    type Buffer: SequenceBuffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation>;
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer);
}
pub trait SequenceBuffer {
    fn buffered_sequence(&self) -> &[u8];
}

/// `PatternLocation` holds the index of a pattern within a target.
/// 
/// The positions within `PatternLocation` should be sorted in ascending order. In general,
/// these positions are automatically sorted when searching for an index within a target.
/// **Note that the algorithm does not perform reordering**.
/// 
/// Each position's value is restricted to the bounds of a `u32`, limiting the range of each position.
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
