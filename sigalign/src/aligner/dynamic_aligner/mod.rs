use sigalign_core::{
    reference::{
        Reference as RawReference,
        PatternIndex, SequenceStorage,
    },
    aligner::{
        AlignmentRegulator,
        local::{LocalAligner, LocalWithLimitAligner},
        semi_global::{SemiGlobalAligner, SemiGlobalWithLimitAligner},
    },
    results::QueryAlignment,
};

#[derive(Clone)]
pub enum DynamicAligner {
    Local(LocalAligner),
    LocalWithLimit(LocalWithLimitAligner),
    SemiGlobal(SemiGlobalAligner),
    SemiGlobalWithLimit(SemiGlobalWithLimitAligner),
}

impl DynamicAligner {
    /// Align a query to the reference which is totally compatible with `sigalign-core`.
    pub fn alignment<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        query: &[u8],
        reference: &RawReference<I, S>,
        sequence_buffer: &mut S::Buffer,
        sorted_target_indices: &[u32],
    ) -> QueryAlignment {
        match self {
            Self::Local(v) => v.alignment(
                query,
                reference,
                sequence_buffer,
                sorted_target_indices,
            ),
            Self::LocalWithLimit(v) => v.alignment(
                query,
                reference,
                sequence_buffer,
                sorted_target_indices,
            ),
            Self::SemiGlobal(v) => v.alignment(
                query,
                reference,
                sequence_buffer,
                sorted_target_indices,
            ),
            Self::SemiGlobalWithLimit(v) => v.alignment(
                query,
                reference,
                sequence_buffer,
                sorted_target_indices,
            ),
        }
    }
    pub fn regulator(&self) -> &AlignmentRegulator {
        match self {
            Self::Local(v) => v.regulator(),
            Self::LocalWithLimit(v) => v.regulator(),
            Self::SemiGlobal(v) => v.regulator(),
            Self::SemiGlobalWithLimit(v) => v.regulator(),
        }
    }
}

impl DynamicAligner {
    pub fn new_local(regulator: AlignmentRegulator) -> Self {
        let aligner = LocalAligner::new(regulator);
        Self::Local(aligner)
    }
    pub fn new_semi_global(regulator: AlignmentRegulator) -> Self {
        let aligner = SemiGlobalAligner::new(regulator);
        Self::SemiGlobal(aligner)
    }
}
