use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder, Serializable,
};

impl<SR> Reference<SR> where
    SR: SequenceProvider + ReverseComplement,
{
    pub fn is_reverse_complement(&self, record_index: usize) -> bool {
        self.sequence_provider.is_reverse_complement(record_index)
    }
}

// For reverse complementary for nucleotide sequence
pub trait ReverseComplement {
    fn is_reverse_complement(&self, record_index: usize) -> bool;
}
