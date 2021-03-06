use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

// Reference interface implementation
impl<S> ReferenceInterface for Reference<S> where
    S: SequenceProvider,
{
    type Buffer = S::Buffer;

    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.pattern_finder.locate_in_record_search_range(pattern, &self.target_record_index)
    }
    fn get_buffer(&self) -> Self::Buffer {
        self.sequence_provider.get_buffer()
    }
    fn fill_sequence_buffer(&self, record_index: usize, buffer: &mut Self::Buffer) {
        self.sequence_provider.fill_sequence_buffer(record_index, buffer)
    }
    fn searchable(&self, pattern: Sequence) -> bool {
        self.sequence_type.searchable(pattern)
    }
}
