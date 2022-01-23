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
use super::{
    Serializable,
    LabelProvider,
};

// Basic sequence providers implementations
// mod in_memory;
// mod indexed_fasta;


/// Provide sequence information
pub trait SequenceProvider {
    type Buffer: SequenceBuffer;

    fn total_record_count(&self) -> usize;
    fn get_buffer(&self) -> Self::Buffer;
    fn sequence_of_record(&self, record_index: usize, buffer: &mut Self::Buffer);
    fn get_joined_sequence(&self) -> JoinedSequence {
        let total_record_count = self.total_record_count();
        let mut record_boundary_positions = Vec::with_capacity(total_record_count + 1);
        record_boundary_positions.push(0);
        let mut accumulated_length = 0;

        let mut sequence_buffer = self.get_buffer();

        let bytes: Vec<u8> = (0..total_record_count).map(|record_index| {
            self.sequence_of_record(record_index, &mut sequence_buffer);
            let record_sequence = sequence_buffer.request_sequence();
            accumulated_length += record_sequence.len() as u64;
            record_boundary_positions.push(accumulated_length);

            record_sequence.to_vec()
        }).flatten().collect();
        
        JoinedSequence::new(bytes, record_boundary_positions)
    }
}
