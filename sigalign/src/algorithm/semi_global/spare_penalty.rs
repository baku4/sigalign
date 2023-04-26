use crate::core::regulators::{
    Penalty, Cutoff, MinPenaltyForPattern, PREC_SCALE,
};
#[derive(Debug, Clone)]

// FIXME:
pub struct SemiGlobalSparePenaltyCalculator {
    // For right spare penalty
    fj: Vec<i64>,
    gjrev: Vec<u32>,
    vars_for_right: (u32, u32),
    last_pattern_index: u32, // This field is needed to be changed by query
    // For left spare penalty
    vars_for_left: (u32, u32, u32, u32),
    min_spare_penalty: u32,
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
        //   - f(j): using left penalty delta by pattern index
        let kd: i64 = (pattern_size * maximum_scaled_penalty_per_length) as i64;
        let scaled_penalty_delta_by_parity = [
            kd - (min_penalty_for_pattern.odd * PREC_SCALE) as i64, // even: add odd for next
            kd - (min_penalty_for_pattern.even * PREC_SCALE) as i64, // odd: add even for next
        ];
        let mut left_scaled_penalty_delta = 0;
        let fj: Vec<i64> = (0..max_pattern_count).map(|j| {
            let v = penalties.e as i64 * left_scaled_penalty_delta;
            left_scaled_penalty_delta += scaled_penalty_delta_by_parity[(j % 2) as usize];
            v
        }).collect();
        //   - g(j_rev): using the reversed pattern index
        //     j_rev: \bar{j} - j (last pattern index - pattern index)
        let gjrev: Vec<u32> = (0..max_pattern_count).map(|j_rev| {
            maximum_scaled_penalty_per_length * penalties.e
            * (
                pattern_size * (j_rev + 2) - 1
            )
        }).collect();
        let a_1 = maximum_scaled_penalty_per_length * penalties.o;
        let b_1 = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;


        // (1) For left spare penalty
        // ( a * penalty delta + b * pattern index - c ) / d
        let a_2 = penalties.e;
        let b_2 = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let c_2 = a_1;
        let d_2 = b_1;

        Self {
            fj,
            gjrev,
            vars_for_right: (a_1, b_1),
            last_pattern_index: 0,
            vars_for_left: (a_2, b_2, c_2, d_2),
            min_spare_penalty: penalties.o,
        }
    }
    pub fn get_right_spare_penalty(
        &self,
        pattern_index: u32,
    ) -> u32 {
        u32::max(
            ((
                self.fj[pattern_index as usize]
                + self.gjrev[(self.last_pattern_index - pattern_index) as usize] as i64
                - self.vars_for_right.0 as i64
            ) / self.vars_for_right.1 as i64 ) as u32,
            self.min_spare_penalty,
        )
    }
    pub fn get_left_spare_penalty(
        &self,
        right_penalty_delta: i64,
        pattern_index: u32,
    ) -> u32 {
        // TODO: Can be more faster?
        i64::max(
            (
                self.vars_for_left.0 as i64 * right_penalty_delta
                + self.vars_for_left.1 as i64 * pattern_index as i64
                - self.vars_for_left.2 as i64
            ) / self.vars_for_left.3 as i64,
            self.min_spare_penalty as i64
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
