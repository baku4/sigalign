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
    right_spare_penalty_by_pattern_index_from_the_right: Vec<u32>,
    last_pattern_index: u32, // This field is needed to be changed by query
    // (a, b, c, d, min_value) for
    // f(right penalty delta, pattern_index)
    left_coefficients_by_variable: (u32, u32, u32, u32, u32),
}
impl SparePenaltyCalculator {
    pub fn new(
        penalties: &Penalty,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) -> Self {
        // (1) For right spare penalty
        // Assuming
        //   - right remained query length
        //      = pattern_size * (max_pattern_count - pattern_index) + (pattern_size - 1)
        //   - left penalty delta
        //      =  (pattern_size - 1) * maximum_scaled_penalty_per_length
        //
        // x: pattern index from the right

        // f(x) = (a*x + b) / c
        // TODO: Can be more faster?

        let a_1 = maximum_scaled_penalty_per_length
            * penalties.e * pattern_size;
        let b_1 = maximum_scaled_penalty_per_length * (
            penalties.e * (3 * pattern_size - 2)
            - penalties.o
        );
        let c_1 = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

        // TODO: last index (first pattern)'s left penalty delta is 0
        let fx: Vec<u32> = (0..=max_pattern_count).into_iter().map(|x| {
            u32::max(
                (a_1*x + b_1) / c_1,
                penalties.o,
            )
        }).collect();

        // (2) For left spare penalty
        let a_2 = penalties.e;
        let b_2 = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let c_2 = maximum_scaled_penalty_per_length * penalties.o;
        let d = c_1;

        Self {
            right_spare_penalty_by_pattern_index_from_the_right: fx,
            last_pattern_index: max_pattern_count - 1,
            left_coefficients_by_variable: (a_2, b_2, c_2, d, penalties.o),
        }
    }
    pub fn get_right_spare_penalty(
        &self,
        pattern_index: u32,
    ) -> u32 {
        self.right_spare_penalty_by_pattern_index_from_the_right[
            (self.last_pattern_index - pattern_index) as usize
        ]
    }
    pub fn get_left_spare_penalty(
        &self,
        right_penalty_delta: i64,
        pattern_index: u32,
    ) -> u32 {
        // TODO: Can be more faster?
        u32::max(
            ((
                self.left_coefficients_by_variable.0 as i64 * right_penalty_delta
                + self.left_coefficients_by_variable.1 as i64 * pattern_index as i64
                + self.left_coefficients_by_variable.2 as i64
            ) / self.left_coefficients_by_variable.3 as i64) as u32,
            self.left_coefficients_by_variable.4
        )
    }
    pub fn change_total_pattern_count(
        &mut self,
        last_pattern_index: u32,
    ) {
        self.last_pattern_index = last_pattern_index;
    }
}

pub fn get_right_spare_penalty_by_pattern_index_from_the_right(
    penalties: &Penalty,
    maximum_scaled_penalty_per_length: u32,
    pattern_size: u32,
    max_pattern_count: u32,
) -> Vec<u32> {
    // Right spare penalty

    // Assuming
    //   - right remained query length
    //      = pattern_size * (max_pattern_count - pattern_index) + (pattern_size - 1)
    //   - left penalty delta
    //      =  (pattern_size - 1) * maximum_scaled_penalty_per_length
    //
    // x: pattern index from the right

    // f(x) = (a*x + b) / c
    // TODO: Can be more faster?

    let a = maximum_scaled_penalty_per_length
        * penalties.e * pattern_size;
    let b = maximum_scaled_penalty_per_length * (
        penalties.e * (3 * pattern_size - 2)
        - penalties.o
    );
    let c = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

    let fx: Vec<u32> = (0..=max_pattern_count).into_iter().map(|x| {
        u32::max(
            (a*x + b) / c,
            penalties.o,
        )
    }).collect();

    // // TODO: last index (first pattern)'s left penalty delta is 0
    fx
}

pub struct DepSparePenaltyCalculator {
    left_additive_spare_penalty_by_pattern_index: Vec<u32>,
    left_additive_spare_penalty_by_pattern_count: Vec<u32>,
    right_spare_penalty_by_remained_pattern_count: Vec<u32>,
    total_pattern_count: u32, // This field is needed to be changed by query
}
impl DepSparePenaltyCalculator {
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
