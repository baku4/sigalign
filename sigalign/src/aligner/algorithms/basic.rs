use sigalign_core::aligner::{
    AlignmentRegulator,
    local::LocalAligner,
    semi_global::SemiGlobalAligner,
};
use crate::{
    Reference,
    reference::DefaultSequenceBuffer,
    results::QueryAlignment,
};
use super::{Algorithm, ParamsError, check_pattern_size};

// Structs
pub struct Local {
    inner: LocalAligner,
}

pub struct SemiGlobal {
    inner: SemiGlobalAligner,
}

// New
fn get_basic_regulator(
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

impl Local {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_basic_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        Ok(Self {
            inner: LocalAligner::new(regulator),
        })
    }
}

impl SemiGlobal {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_basic_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        Ok(Self {
            inner: SemiGlobalAligner::new(regulator),
        })
    }
}

// Implement Algorithm
impl Algorithm for Local {
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

impl Algorithm for SemiGlobal {
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
