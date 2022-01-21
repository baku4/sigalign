use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder,
};

// Reference interface implementation
impl<S> ReferenceInterface for Reference<S> where
    S: SequenceProvider,
{
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.pattern_finder.locate_in_record_search_range(pattern, &self.target_record_index)
    }
    fn sequence_of_record(&self, record_index: usize) -> Sequence {
        self.sequence_provider.sequence_of_record(record_index)
    }
    fn searchable(&self, pattern: Sequence) -> bool {
        self.sequence_type.searchable(pattern)
    }
}
