use sigalign_core::aligner::{
    AlignmentRegulator,
    local::LocalWithLimitAligner,
    semi_global::SemiGlobalWithLimitAligner,
};
use crate::{
    Reference,
    reference::DefaultSequenceBuffer,
    results::QueryAlignment,
};
use super::{Algorithm, ParamsError, check_pattern_size};

// Structs
#[derive(Clone)]
pub struct LocalWithLimit {
    inner: LocalWithLimitAligner,
}

#[derive(Clone)]
pub struct SemiGlobalWithLimit {
    inner: SemiGlobalWithLimitAligner,
}

// New
fn get_regulator(
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    minimum_length: u32,
    maximum_penalty_per_length: f32,
) -> Result<AlignmentRegulator, ParamsError> {
    let regulator = AlignmentRegulator::new(
        mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length
    )?;
    check_pattern_size(&regulator)?;
    Ok(regulator)
}

impl LocalWithLimit {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
        limit: u32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        Ok(Self {
            inner: LocalWithLimitAligner::new(regulator, limit),
        })
    }
}

impl SemiGlobalWithLimit {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
        limit: u32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        Ok(Self {
            inner: SemiGlobalWithLimitAligner::new(regulator, limit),
        })
    }
}

// Implement Algorithm
impl Algorithm for LocalWithLimit {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment {
        self.inner.align(
            query,
            reference.as_ref(),
            sequence_buffer,
            reference.get_full_sorted_target_indices(),
        )
    }
    fn regulator(&self) -> &AlignmentRegulator {
        self.inner.regulator()
    }
}

impl Algorithm for SemiGlobalWithLimit {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment {
        self.inner.align(
            query,
            reference.as_ref(),
            sequence_buffer,
            reference.get_full_sorted_target_indices(),
        )
    }
    fn regulator(&self) -> &AlignmentRegulator {
        self.inner.regulator()
    }
}

// Debug
impl std::fmt::Debug for LocalWithLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalWithLimit")
            .field("mismatch_penalty", &self.regulator().get_mismatch_penalty())
            .field("gap_open_penalty", &self.regulator().get_gap_open_penalty())
            .field("gap_extend_penalty", &self.regulator().get_gap_extend_penalty())
            .field("minimum_length", &self.regulator().get_minimum_length())
            .field("maximum_penalty_per_length", &self.regulator().get_maximum_penalty_per_length())
            .field("limit", &self.inner.limit())
            .finish()
    }
}
impl std::fmt::Debug for SemiGlobalWithLimit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemiGlobalWithLimit")
            .field("mismatch_penalty", &self.regulator().get_mismatch_penalty())
            .field("gap_open_penalty", &self.regulator().get_gap_open_penalty())
            .field("gap_extend_penalty", &self.regulator().get_gap_extend_penalty())
            .field("minimum_length", &self.regulator().get_minimum_length())
            .field("maximum_penalty_per_length", &self.regulator().get_maximum_penalty_per_length())
            .field("limit", &self.inner.limit())
            .finish()
    }
}
