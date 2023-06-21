use crate::results::{
    labeled::{
        LabeledAlignmentResult,
    },
};
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
    extensions::LabelStorage,
};
use super::{Aligner, AlignmentMode, AllocationStrategy};

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    // Query
    pub fn align_query_labeled<I, S> (
        &mut self,
        reference: &Reference<I, S>,
        query: &[u8],
    ) -> LabeledAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
    {
        let mut sequence_buffer = reference.get_sequence_buffer();
        let result = self.alignment(reference, &mut sequence_buffer, query);
        result.to_labeled_result_unchecked(reference)
    }
}