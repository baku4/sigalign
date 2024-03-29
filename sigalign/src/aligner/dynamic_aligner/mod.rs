use sigalign_core::{
    reference::{
        Reference as RawReference,
        PatternIndex, SequenceStorage,
    },
    aligner::{
        Aligner as RawAligner,
        LocalAligner, LocalWithLimitAligner,
        SemiGlobalAligner, SemiGlobalWithLimitAligner, AlignmentRegulator,
    },
    results::AlignmentResult,
};
use sigalign_impl::allocation_strategy::LinearStrategy;

#[derive(Clone)]
pub enum DynamicAligner {
    Local(LocalAligner<LinearStrategy>),
    LocalWithLimit(LocalWithLimitAligner<LinearStrategy>),
    SemiGlobal(SemiGlobalAligner<LinearStrategy>),
    SemiGlobalWithLimit(SemiGlobalWithLimitAligner<LinearStrategy>),
}

impl RawAligner for DynamicAligner {
    fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &RawReference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
        query: &[u8],
    ) -> AlignmentResult {
        match self {
            Self::Local(v) => v.alignment(reference, sequence_buffer, sorted_target_indices, query),
            Self::LocalWithLimit(v) => v.alignment(reference, sequence_buffer, sorted_target_indices, query),
            Self::SemiGlobal(v) => v.alignment(reference, sequence_buffer, sorted_target_indices, query),
            Self::SemiGlobalWithLimit(v) => v.alignment(reference, sequence_buffer, sorted_target_indices, query),
        }
    }
}

impl DynamicAligner {
    pub fn new_local(regulator: AlignmentRegulator) -> Self {
        let local_aligner = LocalAligner::new(regulator);
        Self::Local(local_aligner)
    }
}
