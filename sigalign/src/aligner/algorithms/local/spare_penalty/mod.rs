use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
    }
};
use super::{PosTable, AnchorIndex, AnchorPosition, TraversedAnchorDep};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use ahash::AHashSet;

#[inline]
pub fn get_left_spare_penalty_by_pattern_index(
    penalties: &Penalty,
    maximum_scaled_penalty_per_length: u32,
    pattern_size: u32,
    pattern_count: u32,
) -> Vec<u32> {
    // h(x) = (a*x + b) / c
    let a = maximum_scaled_penalty_per_length * penalties.e;
    let b = maximum_scaled_penalty_per_length * (
        // (k-1)*d as penalty delta
        penalties.e * pattern_size - penalties.o - penalties.e
    );
    let c = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

    let mut left_spare_penalty_by_pattern_index: Vec<u32> = (0..pattern_count).into_iter().map(|x| {
        u32::max(
            (a * x + b) / c,
            penalties.o,
        )
    }).collect();
    left_spare_penalty_by_pattern_index[0] = 0;

    left_spare_penalty_by_pattern_index
}
