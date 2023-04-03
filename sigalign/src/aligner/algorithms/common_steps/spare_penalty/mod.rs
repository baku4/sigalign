use crate::core::{
    regulators::{
	    Penalty, PREC_SCALE, Cutoff,
    },
};

// TODO: Remove the unnecessary casting (as) 
pub fn calculate_spare_penalty(
    scaled_penalty_delta_of_other_side: i64,
    anchor_size: u32,
    query_length_this_side: u32,
    record_length_this_side: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
) -> u32 {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * scaled_penalty_delta_of_other_side
            + cutoff.maximum_penalty_per_scale as i64 * (
                (
                    penalties.e as i64 * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    ) as i64
                ) - penalties.o as i64
            )
        ) / (
            PREC_SCALE  as i64 * penalties.e  as i64 - cutoff.maximum_penalty_per_scale  as i64
        ) + 1
    ) as u32
}
