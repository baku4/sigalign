use crate::core::{
    ReferenceInterface,
    results::AlignmentResult,
};
use super::{
    AlignerInterface,
    AlignmentRegulator, RegulatorError,
    WaveFrontCache, SingleWaveFrontCache, semi_global_alignment_algorithm,
};

#[derive(Clone)]
pub struct SemiGlobalAligner {
    pub regulator: AlignmentRegulator,
    pub wave_front_cache: SingleWaveFrontCache,
}

impl AlignerInterface for SemiGlobalAligner {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_cache = SingleWaveFrontCache::new(&regulator.penalties, &regulator.cutoff);
        Ok(Self {
            regulator,
            wave_front_cache,
        })
    }
    fn alignment<R: ReferenceInterface>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        self.wave_front_cache.allocate_more_if_necessary(
            query.len() as u32,
            &self.regulator.penalties,
            &self.regulator.cutoff,
        );
        let reference_alignment_result = semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.min_penalty_for_pattern,
            &self.regulator.cutoff,
            &mut self.wave_front_cache.wave_front,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
