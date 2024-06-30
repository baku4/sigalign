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
        let mut results = Vec::new();
        
        let mut start = 0;
        while start + self.segment_size as usize <= query.len() {
            let slice = &query[start..start + self.segment_size as usize];
            let mut alignment = self.inner.align(
                slice,
                reference.as_ref(),
                sequence_buffer,
                reference.get_full_sorted_target_indices(),
            );
            adjust_positions(&mut alignment, start);
            results.append(&mut alignment.0);

            start += self.sliding_size as usize;
        }

        QueryAlignment(results)
    }
}

impl Algorithm for SemiGlobalWithChunk {
    fn align(
        &mut self,
        query: &[u8],
        reference: &Reference,
        sequence_buffer: &mut DefaultSequenceBuffer,
    ) -> QueryAlignment {
        let mut results = Vec::new();
        
        let mut start = 0;
        while start + self.segment_size as usize <= query.len() {
            let slice = &query[start..start + self.segment_size as usize];
            let mut alignment = self.inner.align(
                slice,
                reference.as_ref(),
                sequence_buffer,
                reference.get_full_sorted_target_indices(),
            );
            adjust_positions(&mut alignment, start);
            results.append(&mut alignment.0);

            start += self.sliding_size as usize;
        }
        
        QueryAlignment(results)
    }
}

fn adjust_positions(
    alignment: &mut QueryAlignment,
    start: usize,
) {
    alignment.0.iter_mut().for_each(|tgt_aln| {
        tgt_aln.alignments.iter_mut().for_each(|aln| {
            aln.position.query.0 += start as u32;
            aln.position.query.1 += start as u32;
        })
    });
}
