use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{AlignmentCondition, WaveFrontCache, DoubleWaveFrontCache};
use super::local_alignment_algorithm;

pub struct LocalAligner {
    pub condition: AlignmentCondition,
    pub wave_front_cache: DoubleWaveFrontCache,
    sequence_buffer: Vec<u8>,
}

impl AlignerInterface for LocalAligner {
    fn alignment(&mut self, reference: &dyn ReferenceInterface, query: Sequence) -> AlignmentResult {
        let reference_alignment_result = local_alignment_algorithm(
            reference,
            query,
            self.condition.pattern_size,
            &self.condition.penalties,
            &self.condition.cutoff,
            &self.condition.min_penalty_for_pattern,
            &mut self.wave_front_cache.primary_wave_front,
            &mut self.wave_front_cache.secondary_wave_front,
        );

        self.condition.result_of_uncompressed_penalty(reference_alignment_result)
    }
}

impl LocalAligner {
    pub(crate) fn new(condition: AlignmentCondition) -> Self {
        let wave_front_cache = DoubleWaveFrontCache::new(&condition.penalties, &condition.cutoff);
        Self {
            condition,
            wave_front_cache,
            sequence_buffer: Vec::new(),
        }
    }
}