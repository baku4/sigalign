use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    Reference, SequenceProvider,
    AlignmentCondition,
    SemiGlobalAligner, LocalAligner,
    Aligner, Algorithms, AlignerInterface,
};

use std::cmp::{PartialEq, Eq};

impl PartialEq for Aligner {
    fn eq(&self, other: &Self) -> bool {
        let (self_is_semi_global, self_alignment_condition) = self.is_semi_global_and_condition();
        let (other_is_semi_global, other_alignment_condition) = other.is_semi_global_and_condition();

        (self_is_semi_global == other_is_semi_global) && (self_alignment_condition == other_alignment_condition)
    }
}

impl Eq for Aligner {}

impl Aligner {
    fn is_semi_global_and_condition(&self) -> (bool, &AlignmentCondition) {
        match &self.algorithms {
            Algorithms::SemiGlobal(aligner) => {
                (false, &aligner.condition)
            },
            Algorithms::Local(aligner) => {
                (false, &aligner.condition)
            },
        }
    }
}