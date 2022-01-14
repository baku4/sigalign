use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	ReferenceAlignmentResult, RecordAlignmentResult, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType,
    Sequence,
    ReferenceInterface, PatternLocation,
    AlignerInterface,
};
use super::{WaveFront};

struct WaveFrontCache {
    
}

// WaveFront Allocation
impl WaveFront {
    const QUERY_LEN_INC_UNIT: usize = 100;

    fn new_with_spacious_query_length(
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> (usize, Self) {
        let to_allocate_query_length = Self::QUERY_LEN_INC_UNIT;
        let max_score = Self::safe_max_score_from_length(penalties, cutoff, to_allocate_query_length);

        (to_allocate_query_length, WaveFront::new_allocated(penalties, max_score))
    }
    fn upper_safe_max_score_from_length(
        query_length: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> usize {
        let to_allocate_query_length = ((query_length / Self::QUERY_LEN_INC_UNIT) + 1) * Self::QUERY_LEN_INC_UNIT;
        let safe_max_score = Self::safe_max_score_from_length(penalties, cutoff, to_allocate_query_length);

        safe_max_score
    }
    fn safe_max_score_from_length(
        penalties: &Penalties,
        cutoff: &Cutoff,
        query_length: usize,
    ) -> usize {
        (
            cutoff.maximum_penalty_per_scale * (
                penalties.e * query_length - penalties.o
            )
        ) / (
            PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) + 1
    }
}