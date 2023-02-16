pub mod regulators;
pub mod results;

mod sequence_length;
pub use sequence_length::SeqLen;

// Reference
pub trait ReferenceInterface<L: SeqLen> {
    type Buffer: SequenceBuffer;

    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation<L>>;
    fn get_buffer(&self) -> Self::Buffer;
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer);
    fn is_indexed(&self, query: &[u8]) -> bool;
}
pub trait SequenceBuffer {
    fn request_sequence(&self) -> &[u8];
}
#[derive(Debug)]
pub struct PatternLocation<L: SeqLen> {
    pub target_index: u32,
    pub locations: Vec<L>,
}
