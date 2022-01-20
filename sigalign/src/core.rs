// Core data structures
use crate::{Result, error_msg};

mod conditions;
mod result;

pub use conditions::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
pub use result::{ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};

// Sequence
pub type Sequence<'a> = &'a [u8];

// Reference
pub trait ReferenceInterface {
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation>;
    // It should be guaranteed that the record index is within search range.
    fn sequence_of_record(&self, record_index: usize) -> Sequence;
}

#[derive(Debug)]
pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}

// Aligner
pub trait AlignerInterface {
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> where Self: Sized;
    fn alignment(
        &mut self,
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
    ) -> ReferenceAlignmentResult;
}
