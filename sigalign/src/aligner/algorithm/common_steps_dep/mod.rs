use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
};

// Wavefront structure for alignment
mod wave_front;
pub use wave_front::{WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

// Deprecated
mod dwfa;

// Extension
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


use std::collections::HashSet;

pub struct AlignmentHashSet {
    hash_set: HashSet<(usize, AlignmentPosition)>
}

impl AlignmentHashSet {
    pub fn new() -> Self {
        Self {
            hash_set: HashSet::new()
        }
    }
    pub fn insert_and_check_new(&mut self, penalty: usize, alignment_position: AlignmentPosition) -> bool {
        self.hash_set.insert((penalty, alignment_position))
    }
}