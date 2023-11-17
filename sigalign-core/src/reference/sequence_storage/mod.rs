/*!
Provides the `SequenceStorage` and its basic implementations.
*/

pub use crate::core::SequenceBuffer;
use super::ConcatenatedSequenceWithBoundaries;
pub trait SequenceStorage {
    type Buffer: SequenceBuffer;

    fn get_buffer(&self) -> Self::Buffer;
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer);
    fn num_targets(&self) -> u32;

    fn get_concatenated_sequence_with_boundaries(&self) -> ConcatenatedSequenceWithBoundaries {
        let num_targets = self.num_targets();
        let mut boundaries = Vec::with_capacity(num_targets as usize + 1);
        boundaries.push(0);
        let mut accumulated_length = 0;

        let mut buffer = self.get_buffer();
        let mut concatenated_sequence = Vec::new();
        for target_index in 0..num_targets {
            self.fill_buffer(target_index, &mut buffer);
            let record_sequence = buffer.buffered_sequence();
            accumulated_length += record_sequence.len() as u64;
            boundaries.push(accumulated_length);
            concatenated_sequence.extend_from_slice(record_sequence)
        }
        
        ConcatenatedSequenceWithBoundaries {
            concatenated_sequence,
            boundaries,
        }
    }
}

pub mod in_memory;
