use crate::core::regulators::{
    Penalty, PREC_SCALE,
};
#[derive(Debug, Clone)]
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
        // ( a * penalty delta + b * pattern index - c ) / d
        let a_2 = penalties.e;
        let b_2 = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let c_2 = maximum_scaled_penalty_per_length * penalties.o;
        let d = c_1;

        Self {
            right_spare_penalty_by_pattern_index_from_the_right: fx,
            last_pattern_index: 0,
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
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) {
        // TODO: Do not make the new struct
        let new = Self::new(penalties, maximum_scaled_penalty_per_length, pattern_size, max_pattern_count);
        *self = new;
    }
    pub fn change_last_pattern_index(
        &mut self,
        last_pattern_index: u32,
    ) {
        self.last_pattern_index = last_pattern_index;
    }
}
