use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceStorage,
};

pub fn calculate_spare_penalty(
    scaled_penalty_margin_of_other_side: i64,
    anchor_size: usize,
    query_length_this_side: usize,
    record_length_this_side: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
) -> usize {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * scaled_penalty_margin_of_other_side
            + cutoff.maximum_penalty_per_scale as i64 * (
                (
                    penalties.e * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    )
                ) as i64 - penalties.o as i64
            )
        ) / (
            PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) as i64 + 1
    ) as usize
}