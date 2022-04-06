use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, JoinedSequence,
    SequenceType, PatternFinder,
};

// Traits to implement
pub use super::{
    Serializable, SizeAware, Divisible,
    LabelProvider, ReverseComplement,
};

// Basic sequence providers implementations
mod in_memory;
pub use in_memory::{InMemoryProvider, InMemoryRcProvider};
mod indexed_fasta;
pub use indexed_fasta::{IndexedFastaProvider, IndexedFastaRcProvider};
// Utils for sequence provider
mod util;


/// Provide sequence information
pub trait SequenceProvider {
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
