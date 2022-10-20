use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceStorage,
    SequenceType, PatternFinder, Serializable,
};

impl<SR> Reference<SR> where
    SR: SequenceStorage + RcStorage,
{
    pub fn is_reverse_complement(&self, record_index: usize) -> bool {
        self.sequence_storage.is_reverse_complement(record_index)
    }
}

// For reverse complementary for nucleotide sequence
/// Storage for reverse complementary (for nucleotide sequence) information of sequences.
pub trait RcStorage {
    fn is_reverse_complement(&self, record_index: usize) -> bool;
}
