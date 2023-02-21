use crate::core::{
    SeqLen,
    regulators::{
	    Penalty, PREC_SCALE, Cutoff,
    },
};

pub fn calculate_spare_penalty<L: SeqLen>(
    scaled_penalty_margin_of_other_side: i64,
    anchor_size: L,
    query_length_this_side: L,
    record_length_this_side: L,
    penalties: &Penalty,
    cutoff: &Cutoff,
) -> usize {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * scaled_penalty_margin_of_other_side
            + cutoff.maximum_penalty_per_scale as i64 * (
                (
                    penalties.e as i64 * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    ).as_i64()
                ) - penalties.o as i64
            )
        ) / (
            PREC_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) as i64 + 1
    ) as usize
}