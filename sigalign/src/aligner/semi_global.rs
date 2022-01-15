use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{AlignmentCondition, WaveFrontCache, SingleWaveFrontCache};
use super::semi_global_alignment_algorithm;

pub struct SemiGlobalAligner {
    condition: AlignmentCondition,
    wave_front_cache: SingleWaveFrontCache,
}

impl AlignerInterface for SemiGlobalAligner {
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_cache = SingleWaveFrontCache::new(&condition.penalties, &condition.cutoff);

        Ok(
            Self {
                condition,
                wave_front_cache,
            }
        )
    }
    fn alignment(&mut self, reference: &mut dyn ReferenceInterface, query: Sequence) -> ReferenceAlignmentResult {
        let reference_alignment_result = semi_global_alignment_algorithm(
            reference,
            query,
            self.condition.pattern_size,
            &self.condition.penalties,
            &self.condition.cutoff,
            &self.condition.min_penalty_for_pattern,
            &mut self.wave_front_cache.wave_front,
        );

        self.condition.decompress_result(reference_alignment_result)
    }
}