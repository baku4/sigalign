use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{AlignmentCondition, WaveFrontCache, SingleWaveFrontCache};
use super::semi_global_alignment_algorithm;

pub struct SemiGlobalAligner {
    pub condition: AlignmentCondition,
    pub wave_front_cache: SingleWaveFrontCache,
}

impl AlignerInterface for SemiGlobalAligner {
    fn alignment(&mut self, reference: &dyn ReferenceInterface, query: Sequence) -> AlignmentResult {
        let reference_alignment_result = semi_global_alignment_algorithm(
            reference,
            query,
            self.condition.pattern_size,
            &self.condition.penalties,
            &self.condition.cutoff,
            &self.condition.min_penalty_for_pattern,
            &mut self.wave_front_cache.wave_front,
        );

        self.condition.result_of_uncompressed_penalty(reference_alignment_result)
    }
}

impl SemiGlobalAligner {
    pub(crate) fn new(condition: AlignmentCondition) -> Self {
        let wave_front_cache = SingleWaveFrontCache::new(&condition.penalties, &condition.cutoff);
        Self {
            condition,
            wave_front_cache,
        }
    }
}
