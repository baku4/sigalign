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
pub struct LocalWithChunk {
    inner: LocalAligner,
    segment_size: u32,
    sliding_size: u32,
}

pub struct SemiGlobalWithChunk {
    inner: SemiGlobalAligner,
    segment_size: u32,
    sliding_size: u32,
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
fn check_size(size: u32) -> Result<(), ParamsError> {
    if size == 0 {
        Err(ParamsError::InvalidValue("Size must be greater than 0.".to_string()))
    } else {
        Ok(())
    }
}

impl LocalWithChunk {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
        segment_size: u32,
        sliding_size: u32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_basic_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        check_size(segment_size)?;
        check_size(sliding_size)?;
        Ok(Self {
            inner: LocalAligner::new(regulator),
            segment_size,
            sliding_size,
        })
    }
}

impl SemiGlobalWithChunk {
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
        segment_size: u32,
        sliding_size: u32,
    ) -> Result<Self, ParamsError> {
        let regulator = get_basic_regulator(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_length, maximum_penalty_per_length)?;
        check_size(segment_size)?;
        check_size(sliding_size)?;
        Ok(Self {
            inner: SemiGlobalAligner::new(regulator),
            segment_size,
            sliding_size,
        })
    }
}

// Implement Algorithm
impl Algorithm for LocalWithChunk {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment {
        query.windows(self.segment_size as usize)
            .step_by(self.sliding_size as usize)
            .for_each(|chunked_query| {
                //
            });
    }
}

impl Algorithm for SemiGlobalWithChunk {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment {
        query.windows(self.segment_size as usize)
            .step_by(self.sliding_size as usize)
            .for_each(|chunked_query| {
                //
            });
    }
}
