// Core data structures

mod conditions;
mod result;

pub use conditions::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
pub use result::{ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};

// Sequence
pub type Sequence<'a> = &'a [u8];

// Reference
pub trait ReferenceInterface {
    fn is_searchable(&self, query: Sequence) -> bool;
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation>;
    fn sequence_of_record(&mut self, record_index: usize) -> Sequence;
}
pub trait LabeledReferenceInterface: ReferenceInterface {
    fn label_of_record(&mut self, record_index: usize) -> &str;
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
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
    ) -> ReferenceAlignmentResult;
}
