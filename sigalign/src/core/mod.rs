mod regulators;
mod results;

// Regulators
pub use regulators::{
    Penalty,
    PREC_SCALE,
    Cutoff,
    MinPenaltyForPattern,
};

// Results
pub use results::{
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};

// Reference
pub trait ReferenceInterface {
    type Buffer: SequenceBuffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation>;
    fn get_buffer(&self) -> Self::Buffer;
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer);
    fn is_indexed(&self, query: &[u8]) -> bool;
}
pub trait SequenceBuffer {
    fn request_sequence(&self) -> &[u8];
}
#[derive(Debug)]
pub struct PatternLocation {
    pub target_index: usize,
    pub locations: Vec<usize>,
}
