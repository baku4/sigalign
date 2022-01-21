use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{
    Reference, SequenceProvider,
    SequenceType, PatternFinder, SizeAwareEncoding,
};

impl<S> Reference<S> where
    S: SequenceProvider,
{
    pub(crate) fn new(
        sequence_type: SequenceType,
        pattern_finder: PatternFinder,
        target_record_index: Vec<u32>,
        sequence_provider: S
    ) -> Self {
        Self {
            sequence_type,
            pattern_finder,
            target_record_index,
            sequence_provider,
        }
    }
}