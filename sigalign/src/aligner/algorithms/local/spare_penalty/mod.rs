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
use super::{AnchorTable, Anchor, AnchorIndex};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use ahash::AHashSet;

pub struct SparePenaltyCalculator {
    left_additive_spare_penalty_by_pattern_index: Vec<u32>,
    left_additive_spare_penalty_by_pattern_count: Vec<u32>,
    right_spare_penalty_by_remained_pattern_count: Vec<u32>,
    total_pattern_count: u32, // This field is needed to be changed by query
}
impl SparePenaltyCalculator {
    pub fn new(
        penalties: &Penalty,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) -> Self {
        // (1) For the left side

        // f + g = (a*PC + b*PI + c) / d
        let a = penalties.e * pattern_size * PREC_SCALE;
        let b = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let c = a - penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length * penalties.o;
        let d = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

        // f(PI) = floor of (b*PI) / d
        // g(PC) = floor of 1 + (a*PC + c) / d
        let mut fx: Vec<u32> = (0..=max_pattern_count).into_iter().map(|v| b*v / d).collect();
        let mut gx: Vec<u32> = (0..=max_pattern_count).into_iter().map(|v| ((a*v + c) / d) + 1).collect();
        fx[0] = 0;
        gx[0] = 0;

        // (2) For the right side

        // Assuming
        //   - right query length = remained_pattern_count * pattern_size + pattern_size - 1
        //   - left penalty delta = (pattern_size - 1) * maximum_scaled_penalty_per_length

        // h(x): (a*RPC + b) / d
        // d is the  same as before
        let a = maximum_scaled_penalty_per_length * pattern_size * penalties.e;
        let b = penalties.e * (
            maximum_scaled_penalty_per_length + PREC_SCALE
        ) * (pattern_size - 1) - maximum_scaled_penalty_per_length * penalties.o;

        let hx: Vec<u32> = (0..=max_pattern_count).into_iter().map(|v| (a*v+b) / d).collect();

        Self {
            left_additive_spare_penalty_by_pattern_index: fx,
            left_additive_spare_penalty_by_pattern_count: gx,right_spare_penalty_by_remained_pattern_count: hx,
            total_pattern_count: max_pattern_count,
        }
    }
    pub fn change_total_pattern_count(
        &mut self,
        total_pattern_count: u32,
    ) {
        self.total_pattern_count = total_pattern_count;
    }
    #[inline]
    pub fn get_left_spare_penalty(
        &self,
        pattern_index: u32,
        pattern_count: u32,
    ) -> u32 {
        self.left_additive_spare_penalty_by_pattern_index[pattern_index as usize]
        + self.left_additive_spare_penalty_by_pattern_count[pattern_count as usize]
    }
    #[inline]
    pub fn get_right_spare_penalty(
        &self,
        pattern_index: u32,
    ) -> u32 {
        self.right_spare_penalty_by_remained_pattern_count[
            (self.total_pattern_count - pattern_index) as usize
        ]
    }
}

#[inline]
pub fn get_divided_left_spare_penalty(
    penalties: &Penalty,
    maximum_scaled_penalty_per_length: u32,
    pattern_size: u32,
    max_pattern_count: u32,
) -> (Vec<u32>, Vec<u32>) {
    // To calculate
    // h(x) = (a*PC + b*PI + c) / d
    let a = penalties.e * pattern_size * PREC_SCALE;
    let b = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
    let c = a - penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length * penalties.o;
    let d = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

    // f(PC) = floor of 1 + (a*PC + c) / d
    // g(PI) = floor of (b*PC) / d
    let fx: Vec<u32> = (0..max_pattern_count).into_iter().map(|v| ((a*v + c) / d) + 1).collect();
    let gx: Vec<u32> = (0..max_pattern_count).into_iter().map(|v| b*v / d).collect();

    (fx, gx)
}
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
