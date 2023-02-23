use crate::core::{
    ReferenceInterface,
    results::AlignmentResult,
};
use super::{
    Aligner,
    AlignmentRegulator, RegulatorError,
    WaveFrontCache, DoubleWaveFrontCache, local_alignment_algorithm,
};

#[derive(Clone)]
pub struct LocalAligner {
    pub regulator: AlignmentRegulator,
    pub wave_front_cache: DoubleWaveFrontCache,
}

impl Aligner for LocalAligner {
    fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        let regulator = AlignmentRegulator::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_cache = DoubleWaveFrontCache::new(&regulator.penalties, &regulator.cutoff);
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
        let reference_alignment_result = local_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.regulator.pattern_size,
            &self.regulator.penalties,
            &self.regulator.cutoff,
            &mut self.wave_front_cache.primary_wave_front,
            &mut self.wave_front_cache.secondary_wave_front,
        );

        self.regulator.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
