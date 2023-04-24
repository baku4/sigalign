use crate::core::regulators::{
    Penalty, Cutoff, MinPenaltyForPattern, PREC_SCALE,
};
#[derive(Debug, Clone)]

// FIXME:
pub struct SemiGlobalSparePenaltyCalculator {
    right_spare_penalty_by_pattern_index_from_the_right: Vec<u32>,
    last_pattern_index: u32, // This field is needed to be changed by query
    // (a, b, c, d, min_value) for
    // f(right penalty delta, pattern_index)
    left_coefficients_by_variable: (u32, u32, u32, u32, u32),
}
impl SemiGlobalSparePenaltyCalculator {
    pub fn new(
        penalties: &Penalty,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) -> Self {
        // (1) For right spare penalty
        //   - Left penalty delta
        let kd = (pattern_size * maximum_scaled_penalty_per_length) as i64;
        let pd_by_parity = [
            kd - (min_penalty_for_pattern.odd * PREC_SCALE) as i64, // even: add odd for next
            kd - (min_penalty_for_pattern.even * PREC_SCALE) as i64, // odd: add even for next
        ];
        //  - ( a*left penalty delta + b*remained pattern index - c ) / d
        let a_1 = penalties.e as i64;
        let b_1 = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        // b * (k-1) - d' * p_o
        let c_1 = {
            (b_1 * (pattern_size-1)) as i64
            - (maximum_scaled_penalty_per_length * penalties.o) as i64
        };
        let d_1 = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

        let last_pattern_index = max_pattern_count - 1;

        let mut left_penalty_delta  = 0;
        let fx: Vec<i64> = (0..=max_pattern_count).map(|j| {
            (
                (a_1 * left_penalty_delta)
                + (b_1 * pattern_size * (max_pattern_count - j)) as i64
                + c_1
            ) / d_1 as i64
        }).collect();

        // (2) For left spare penalty
        // ( a * penalty delta + b * pattern index - c ) / d
        let a_2 = penalties.e;
        let b_2 = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let c_2 = maximum_scaled_penalty_per_length * penalties.o;
        let d_2 = d_1;

        Self {
            right_spare_penalty_by_pattern_index_from_the_right: Vec::new(),
            last_pattern_index: 0,
            left_coefficients_by_variable: (a_2, b_2, c_2, d_2, penalties.o),
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
        i64::max(
            (
                self.left_coefficients_by_variable.0 as i64 * right_penalty_delta
                + self.left_coefficients_by_variable.1 as i64 * pattern_index as i64
                - self.left_coefficients_by_variable.2 as i64
            ) / self.left_coefficients_by_variable.3 as i64,
            self.left_coefficients_by_variable.4 as i64
        ) as u32
    }
    pub fn allocate(
        &mut self,
        penalties: &Penalty,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) {
        // TODO: Do not make the new struct
        let new = Self::new(
            penalties,
            min_penalty_for_pattern,
            maximum_scaled_penalty_per_length,
            pattern_size,
            max_pattern_count,
        );
        *self = new;
    }
    pub fn change_last_pattern_index(
        &mut self,
        last_pattern_index: u32,
    ) {
        self.last_pattern_index = last_pattern_index;
    }
}

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
            + cutoff.maximum_scaled_penalty_per_length as i64 * (
                (
                    penalties.e as i64 * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    ) as i64
                ) - penalties.o as i64
            )
        ) / (
            PREC_SCALE  as i64 * penalties.e  as i64 - cutoff.maximum_scaled_penalty_per_length  as i64
        ) + 1
    ) as u32
}
