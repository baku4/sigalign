use super::{
	AlignmentResult,
    Sequence,
    Reference, SequenceStorage,
    AlignerInterface,
};
use super::{AlignmentCondition, WaveFrontCache, SingleWaveFrontCache};
use super::semi_global_alignment_algorithm;

#[derive(Clone)]
pub struct SemiGlobalAligner {
    pub condition: AlignmentCondition,
    pub wave_front_cache: SingleWaveFrontCache,
}

impl AlignerInterface for SemiGlobalAligner {
    fn new(condition: AlignmentCondition) -> Self {
        let wave_front_cache = SingleWaveFrontCache::new(&condition.penalties, &condition.cutoff);
        Self {
            condition,
            wave_front_cache,
        }
    }
    fn alignment<S>(&mut self, reference: &Reference<S>, sequence_buffer: &mut S::Buffer, query: Sequence) -> AlignmentResult where
        S: SequenceStorage,
    {
        self.wave_front_cache.allocate_more_if_necessary(query.len(), &self.condition.penalties, &self.condition.cutoff);
        let reference_alignment_result = semi_global_alignment_algorithm(
            reference,
            sequence_buffer,
            query,
            self.condition.pattern_size,
            &self.condition.penalties,
            &self.condition.min_penalty_for_pattern,
            &self.condition.cutoff,
            &mut self.wave_front_cache.wave_front,
        );

        self.condition.result_of_uncompressed_penalty(reference_alignment_result)
    }
}
