// Core data structures
use crate::{Result, error_msg};

mod conditions;
mod result;

pub use conditions::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
pub use result::{AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase};

// Sequence
pub type Sequence<'a> = &'a [u8];

// Reference
pub trait ReferenceInterface {
    type Buffer: SequenceBuffer;

    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation>;
    fn get_buffer(&self) -> Self::Buffer;
    fn sequence_of_record(&self, record_index: usize, buffer: &mut Self::Buffer);
    fn searchable(&self, query: Sequence) -> bool;
}
pub trait SequenceBuffer {
    fn request_sequence(&mut self) -> &[u8];
}

#[derive(Debug)]
pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}
