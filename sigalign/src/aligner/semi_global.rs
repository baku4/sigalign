use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{WaveFront};
use super::{AlignmentCondition};
use std::fmt;

pub struct SemiGlobalAligner {
    condition: AlignmentCondition,
    allocated_query_length: usize,
    wave_front: WaveFront,
}

impl SemiGlobalAligner {
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let spacious_new
    }
}

impl AlignerInterface for SemiGlobalAligner {
    fn alignment(&mut self, reference: &mut dyn ReferenceInterface, query: Sequence) -> ReferenceAlignmentResult {
        
    }
}