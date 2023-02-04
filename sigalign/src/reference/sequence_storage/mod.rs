/*!
The customizable storage of sequences inside the [Reference].

- `SequenceStorage` is storage of sequences.

- The record means one sequence and its additional information such as label.

- The basically implemented [InMemoryStorage] is recommended in most cases.
*/

// Re-export
pub use crate::core::SequenceBuffer;
pub use super::JoinedSequence;

use super::{
    Result,
};

// Traits to implement
pub use super::{
    Serialize, EstimateSize, Divide,
    LabelStorage, RcStorage,
};

// Basic sequence storages implementations
mod in_memory;
pub use in_memory::{InMemoryStorage, InMemoryRcStorage};
mod indexed_fasta;
pub use indexed_fasta::{IndexedFastaStorage, IndexedFastaRcStorage};

/**
Storage for alignment target sequences

- `SequenceStorage` requires `Buffer` and three methods.
    1. `Buffer`
        * Buffer implements [SequenceBuffer]
        * [SequenceBuffer] needs one method: `request_sequence`.
            * `request_sequence` returns pointer to byte of sequence in `Buffer`.
    2. Required methods
        1. `total_record_count`
            * The number of records in this storage.
        2. `get_buffer`
            * Returns empty `Buffer` of `SequenceStorage`.
        3. `fill_sequence_buffer`
            * Fills `Buffer` with sequence of record index.

- Method of `get_joined_sequence` can be overrode for better performance.
    - [JoinedSequence] is required to create index to build [Reference].
    - By default, this method is implemented by summing up each sequence.
*/
pub trait SequenceStorage {
    type Buffer: SequenceBuffer;

    fn total_record_count(&self) -> usize;
    fn get_buffer(&self) -> Self::Buffer;
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer);
    fn get_joined_sequence(&self) -> JoinedSequence {
        let total_record_count = self.total_record_count();
        let mut record_boundary_positions = Vec::with_capacity(total_record_count + 1);
        record_boundary_positions.push(0);
        let mut accumulated_length = 0;

        let mut sequence_buffer = self.get_buffer();

        let mut bytes = Vec::new();
        for record_index in 0..total_record_count {
            self.fill_sequence_buffer(record_index, &mut sequence_buffer);
            let record_sequence = sequence_buffer.request_sequence();
            accumulated_length += record_sequence.len() as u64;
            record_boundary_positions.push(accumulated_length);

            bytes.extend_from_slice(record_sequence)
        }
        
        JoinedSequence::new(bytes, record_boundary_positions)
    }
}
