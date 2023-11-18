use crate::results::AlignmentResult;
use crate::reference::{
    Reference, PatternIndex, SequenceStorage,
};
use super::{Aligner, Algorithm, AllocationStrategy};

use thiserror::Error;
#[derive(Debug, Error)]
pub enum AlignmentError {
    #[error("The query has unsupported character of reference")]
    UnsupportedQuery,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl<A, L> Aligner<A, L> where
    A: Algorithm,
    L: AllocationStrategy,
{
    // Query
    pub fn align_query<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        query: &[u8],
        sorted_target_indices: &[u32],
    ) -> AlignmentResult {
        let mut sequence_buffer = reference.get_sequence_buffer();
        self.alignment(reference, &mut sequence_buffer, query, sorted_target_indices)
    }
}

// Features
mod labeled;