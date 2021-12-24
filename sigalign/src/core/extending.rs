use super::{Penalties, Cutoff, PRECISION_SCALE};
use super::Sequence;
use super::{AlignmentOperation, AlignmentType};

mod dwfa;

mod wave_front;

pub use wave_front::{WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

#[derive(Debug, Clone)]
pub struct Extension {
    pub penalty: usize,
    pub length: usize,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub operations: Vec<AlignmentOperation>,
}

pub fn calculate_spare_penalty_from_determinant(
    spare_penalty_determinant_of_other_side: i64,
    anchor_size: usize,
    query_length_this_side: usize,
    record_length_this_side: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
) -> usize {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * spare_penalty_determinant_of_other_side
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
