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
    fn searchable(&self, query: Sequence) -> bool;
}

#[derive(Debug)]
pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}

// Aligner
pub trait AlignerInterface {
    fn alignment(
        &mut self,
        reference: &dyn ReferenceInterface,
        query: Sequence,
    ) -> ReferenceAlignmentResult;
}
