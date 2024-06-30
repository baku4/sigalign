use crate::{
    results::QueryAlignment,
    reference::{
        Reference,
        DefaultSequenceBuffer,
    }
};

pub mod algorithms;
use algorithms::Algorithm;

mod debug;

/// An alignment executor.
#[derive(Clone)]
pub struct Aligner<A: Algorithm> {
    algorithm: A,
    sequence_buffer: DefaultSequenceBuffer,
}

impl<A: Algorithm> Aligner<A> {
    /// Create a new aligner.
    pub fn new(algorithm: A) -> Self {
        Self::from(algorithm)
    }
    /// Align a query to a reference.
    pub fn align(&mut self, query: &[u8], reference: &Reference) -> QueryAlignment {
        self.algorithm.align(query, reference, &mut self.sequence_buffer)
    }
}

impl<A: Algorithm> From<A> for Aligner<A> {
    fn from(algorithm: A) -> Self {
        Self {
            algorithm,
            sequence_buffer: Reference::get_sequence_buffer(),
        }
    }
}
