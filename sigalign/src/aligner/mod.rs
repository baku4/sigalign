use crate::{
    results::QueryAlignment,
    reference::{
        Reference,
        DefaultSequenceBuffer,
    }
};

pub mod algorithms;
use algorithms::Algorithm;

#[derive(Clone)]
pub struct Aligner<A: Algorithm> {
    algorithm: A,
    sequence_buffer: DefaultSequenceBuffer,
}

impl<A: Algorithm> Aligner<A> {
    pub fn new(algorithm: A) -> Self {
        Self {
            algorithm,
            sequence_buffer: Reference::get_sequence_buffer(),
        }
    }
    pub fn align(&mut self, query: &[u8], reference: &Reference) -> QueryAlignment {
        self.algorithm.align(query, reference, &mut self.sequence_buffer)
    }
}

// Combining all aligners
// (local, local_with_limit, semi_global, semi_global_with_limit)
// into one enum.
// mod dynamic_aligner;
// use dynamic_aligner::DynamicAligner;

// pub mod basic;
// pub mod with_limit;
// pub mod with_chunk;

// // Build the aligner.
// mod build;
// use build::{Algorithm, AlignerBuildError};

// // Perform alignments.
// mod perform_alignments;

// // TODO: WORKING ON
// mod switch_algorithm;
// mod debug;

// /// An alignment executor.
// #[derive(Clone)]
// struct Aligner {
//     dynamic_aligner: DynamicAligner,
//     sequence_buffer: InMemoryBuffer,
// }
