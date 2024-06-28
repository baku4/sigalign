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

fn check_limit(limit: u32) -> Result<(), ParamsError> {
    if limit == 0 {
        Err(ParamsError::InvalidValue("Limit must be greater than 0.".to_string()))
    } else {
        Ok(())
    }
}

// Structs
pub struct LocalWithLimit {
    inner: LocalWithLimitAligner,
}

pub struct SemiGlobalWithLimit {
    inner: SemiGlobalWithLimitAligner,
}

// New
fn get_regulator_with_limit(
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    minimum_length: u32,
    maximum_penalty_per_length: f32,
    limit: u32,
) -> Result<AlignmentRegulator, ParamsError> {
    let regulator = AlignmentRegulator::new(
        mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length
    )?;
    check_pattern_size(&regulator)?;
    check_limit(limit)?;
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
        let regulator = get_regulator_with_limit(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length, limit)?;
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
        let regulator = get_regulator_with_limit(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length, limit)?;
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
}
