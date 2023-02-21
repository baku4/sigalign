use crate::core::{
    SeqLen, ReferenceInterface,
    results::AlignmentResult,
};
use super::{
    AlignmentRegulator, WaveFrontCache, DoubleWaveFrontCache, local_alignment_algorithm
};

#[derive(Clone)]
pub struct LocalAligner {
    pub regulator: AlignmentRegulator,
    pub wave_front_cache: DoubleWaveFrontCache,
}

impl AlignerInterface for LocalAligner {
    fn new(regulator: AlignmentRegulator) -> Self {
        let wave_front_cache = DoubleWaveFrontCache::new(&regulator.penalties, &regulator.cutoff);
        Self {
            regulator,
            wave_front_cache,
        }
    }
    fn alignment<S>(&mut self, reference: &Reference<S>, sequence_buffer: &mut S::Buffer, query: Sequence) -> AlignmentResult where
        S: SequenceStorage,
    {
        self.wave_front_cache.allocate_more_if_necessary(query.len(), &self.condition.penalties, &self.condition.cutoff);
        let reference_alignment_result = local_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.condition.pattern_size,
            &self.condition.penalties,
            &self.condition.cutoff,
            &mut self.wave_front_cache.primary_wave_front,
            &mut self.wave_front_cache.secondary_wave_front,
        );

        self.condition.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
