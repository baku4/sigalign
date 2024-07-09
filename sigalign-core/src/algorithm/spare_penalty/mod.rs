use crate::core::regulators::{
    Penalty, PREC_SCALE,
};
/**
`SparePenaltyCalculator` is a structure designed for rapid spare penalty calculation. 

⚠️ Important: This struct prioritizes speed over safety checks. For safe usage:
1. Instantiate once per regulator using `new`.
2. On increasing the maximum possible length of the query, invoke `precalculate_right_spare_penalty` to update right spare penalties.
3. `change_last_pattern_index` must be invoked for every new query to update the `last_pattern_index`.

Failure to follow these usage rules could lead to unexpected behavior.
*/
#[derive(Debug, Clone)]
pub struct SparePenaltyCalculator {
    precalculated_right_spare_penalty: Vec<u32>,
    last_pattern_index: u32,
    coefficient_for_right: (u32, u32, u32),
    coefficient_for_left: (u32, u32, u32, u32),
    min_penalty: u32,
}

impl SparePenaltyCalculator {
    pub fn new(
        penalties: &Penalty,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
        max_pattern_count: u32,
    ) -> Self {
        let mut spare_penalty_calculator = Self::new_only_with_coefficient(
            penalties,
            maximum_scaled_penalty_per_length,
            pattern_size,
        );
        spare_penalty_calculator.precalculate_right_spare_penalty(max_pattern_count);
        spare_penalty_calculator
    }
    #[inline(always)]
    pub fn get_right_spare_penalty(
        &self,
        pattern_index: u32,
    ) -> u32 {
        self.precalculated_right_spare_penalty[(self.last_pattern_index - pattern_index) as usize]
    }
    #[inline(always)]
    pub fn get_left_spare_penalty(
        &self,
        right_penalty_delta: i32,
        pattern_index: u32,
    ) -> u32 {
        let ce = &self.coefficient_for_left;
        i32::max(
            (
                ce.0 as i32 * right_penalty_delta
                + ce.1 as i32 * pattern_index as i32
                - ce.2 as i32
            ) / ce.3 as i32,
            self.min_penalty as i32
        ) as u32
    }
    #[inline]
    fn new_only_with_coefficient(
        penalties: &Penalty,
        maximum_scaled_penalty_per_length: u32,
        pattern_size: u32,
    ) -> Self {
        // (1) For right spare penalty
        //   - f(x) = (a * x + b) / c
        //   - x: reversed pattern index (= last pattern index - pattern index)
        //   - all coefficient is scaled
        let a = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let b = maximum_scaled_penalty_per_length * (
            penalties.e * (3 * pattern_size - 2) - penalties.o
        );
        let c = penalties.e * PREC_SCALE - maximum_scaled_penalty_per_length;

        // (2) For left spare penalty
        //   - g(y,z) = (d * y + e * z - f) / g
        //   - y: right penalty delta
        //   - z: pattern index
        //   - all coefficient is scaled
        let d = penalties.e;
        let e = maximum_scaled_penalty_per_length * penalties.e * pattern_size;
        let f = maximum_scaled_penalty_per_length * penalties.o;
        // g is same as c
        let g = c;

        Self {
            precalculated_right_spare_penalty: Vec::new(),
            last_pattern_index: 0,
            coefficient_for_right: (a, b, c),
            coefficient_for_left: (d, e, f, g),
            min_penalty: penalties.o,
        }
    }
    #[inline]
    pub fn precalculate_right_spare_penalty(
        &mut self,
        max_pattern_count: u32,
    ) {
        let calculated_pattern_count = self.precalculated_right_spare_penalty.len() as u32;
        let ce = &self.coefficient_for_right;
        for reversed_pattern_index in calculated_pattern_count..max_pattern_count {
            let v = u32::max(
                (ce.0 * reversed_pattern_index + ce.1) / ce.2,
                self.min_penalty,
            );
            self.precalculated_right_spare_penalty.push(v);
        }
    }
    #[inline(always)]
    pub fn change_last_pattern_index(
        &mut self,
        last_pattern_index: u32,
    ) {
        self.last_pattern_index = last_pattern_index;
    }
}
