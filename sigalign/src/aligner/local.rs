use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{AlignmentCondition, WaveFrontCache, DoubleWaveFrontCache};
use super::local_alignment_algorithm;

pub struct LocalAligner {
    condition: AlignmentCondition,
    wave_front_cache: DoubleWaveFrontCache,
}

impl AlignerInterface for LocalAligner {
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        let condition = AlignmentCondition::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?;
        let wave_front_cache = DoubleWaveFrontCache::new(&condition.penalties, &condition.cutoff);

        Ok(
            Self {
                condition,
                wave_front_cache,
            }
        )
    }
    fn alignment(&mut self, reference: &mut dyn ReferenceInterface, query: Sequence) -> ReferenceAlignmentResult {
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

        self.condition.decompress_result(reference_alignment_result)
    }
}