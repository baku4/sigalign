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

pub trait Divisible {
    // Split sequence provider to specific max sized length.
    // If one record exceeds the max length, splitted provider can contain only one of that record.
    fn split_by_max_length(self, max_length: usize) -> Result<Vec<Self>> where Self: Sized;
}
