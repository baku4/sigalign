/*!
The storage of sequences
*/

pub use crate::core::{SequenceBuffer};
use super::{ConcatenatedSequenceWithBoundaries};
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
            let record_sequence = buffer.request_sequence();
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

pub mod implementations;

// - `SequenceStorage` requires `Buffer` and three methods.
//     1. `Buffer`
//         * Buffer implements [SequenceBuffer]
//         * [SequenceBuffer] needs one method: `request_sequence`.
//             * `request_sequence` returns pointer to byte of sequence in `Buffer`.
//     2. Required methods
//         1. `total_record_count`
//             * The number of records in this storage.
//         2. `get_buffer`
//             * Returns empty `Buffer` of `SequenceStorage`.
//         3. `fill_sequence_buffer`
//             * Fills `Buffer` with sequence of record index.
