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
#[derive(Clone)]
pub struct LocalWithChunk {
    inner: LocalAligner,
    segment_size: u32,
    sliding_size: u32,
}

#[derive(Clone)]
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
    fn regulator(&self) -> &AlignmentRegulator {
        self.inner.regulator()
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
    fn regulator(&self) -> &AlignmentRegulator {
        self.inner.regulator()
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

// Debug
impl std::fmt::Debug for LocalWithChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalWithChunk")
            .field("mismatch_penalty", &self.regulator().get_mismatch_penalty())
            .field("gap_open_penalty", &self.regulator().get_gap_open_penalty())
            .field("gap_extend_penalty", &self.regulator().get_gap_extend_penalty())
            .field("minimum_length", &self.regulator().get_minimum_length())
            .field("maximum_penalty_per_length", &self.regulator().get_maximum_penalty_per_length())
            .field("segment_size", &self.segment_size)
            .field("sliding_size", &self.sliding_size)
            .finish()
    }
}
impl std::fmt::Debug for SemiGlobalWithChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SemiGlobalWithChunk")
            .field("mismatch_penalty", &self.regulator().get_mismatch_penalty())
            .field("gap_open_penalty", &self.regulator().get_gap_open_penalty())
            .field("gap_extend_penalty", &self.regulator().get_gap_extend_penalty())
            .field("minimum_length", &self.regulator().get_minimum_length())
            .field("maximum_penalty_per_length", &self.regulator().get_maximum_penalty_per_length())
            .field("segment_size", &self.segment_size)
            .field("sliding_size", &self.sliding_size)
            .finish()
    }
}
