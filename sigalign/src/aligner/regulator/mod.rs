use crate::core::{
    regulators::{
        Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
    },
    results::{
        AlignmentResult, AnchorAlignmentResult,
    },
};
use num::integer;


use thiserror::Error;

#[derive(Error, Debug)]
pub enum RegulatorError {
    #[error("Cutoff is too low to detect the pattern.")]
    LowCutoff,
    #[error("Gap extend penalty only allow positive integer.")]
    GapExtendPenalty,
    #[error("Maximum penalty per length only allow positive value.")]
    NegativeMPpL,

}

const MINIMUM_PATTERN_SIZE: u32 = 4;

pub fn calculate_max_pattern_size(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> u32 {
    let mut n = 1;
    loop { // TODO: Optimize
        let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
        let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
        let max_penalty = (
            (
                (
                    (PREC_SCALE * n * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even))
                )
                + 4 * cutoff.maximum_penalty_per_scale
            ) as f32 / (2 * (n+1) * cutoff.maximum_penalty_per_scale) as f32
        ).ceil() - 2_f32;

        let pattern_size = max_penalty.min(upper_bound);

        if pattern_size >= lower_bound {
            return pattern_size as u32
        }
        n += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlignmentRegulator {
    pub penalties: Penalty,
    pub cutoff: Cutoff,
    pub min_penalty_for_pattern: MinPenaltyForPattern,
    pub gcd_for_compression: u32,
    pub pattern_size: u32,
}

impl AlignmentRegulator {
    /// Generate new aligner.
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Result<Self, RegulatorError> {
        if gap_extend_penalty == 0 {
            return Err(RegulatorError::GapExtendPenalty);
        } else if maximum_penalty_per_length <= 0.0 {
            return Err(RegulatorError::NegativeMPpL);
        }

        let penalties = Penalty::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty);
        let cutoff = Cutoff::new(minimum_aligned_length, maximum_penalty_per_length);

        let aligner = Self::new_with_penalties_and_cutoff(penalties, cutoff);

        let pattern_size = &aligner.pattern_size;
        if *pattern_size < MINIMUM_PATTERN_SIZE {
            return Err(RegulatorError::LowCutoff);
            // error_msg!("Auto calculated pattern size({}) should reach at least {}", pattern_size, MINIMUM_PATTERN_SIZE);
        }

        Ok(aligner)
    }
    fn new_with_penalties_and_cutoff(mut penalties: Penalty, mut cutoff: Cutoff) -> Self {
        let gcd = penalties.gcd_of_penalties();
        penalties.divide_by_gcd(gcd);
        cutoff.divide_by_gcd(gcd);

        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_pattern_size = calculate_max_pattern_size(&cutoff, &min_penalty_for_pattern);
        
        Self {
            penalties,
            cutoff,
            min_penalty_for_pattern,
            gcd_for_compression: gcd,
            pattern_size: max_pattern_size,
        }
    }
    pub fn result_of_uncompressed_penalty(&self, mut reference_alignment_result: AlignmentResult) -> AlignmentResult {
        if self.gcd_for_compression != 1 {
            reference_alignment_result.multiply_gcd(self.gcd_for_compression);
        }
        
        reference_alignment_result
    }
    /// Get penalties
    pub fn get_penalties(&self) -> [u32; 3] {
        [
            self.penalties.x * self.gcd_for_compression,
            self.penalties.o * self.gcd_for_compression,
            self.penalties.e * self.gcd_for_compression,
        ]
    }
    /// Get similarity cutoff
    pub fn get_similarity_cutoff(&self) -> (u32, f32) {
        (
            self.cutoff.minimum_aligned_length,
            (self.cutoff.maximum_penalty_per_scale * self.gcd_for_compression) as f32 / PREC_SCALE as f32,
        )
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> u32 {
        self.pattern_size
    }
}

impl AlignmentResult {
    fn multiply_gcd(&mut self, gcd: u32) {
        self.0.iter_mut().for_each(|record_alignment_result| {
            record_alignment_result.alignments.iter_mut().for_each(|alignment_result| {
                alignment_result.multiply_gcd(gcd);
            })
        })
    }
}

impl AnchorAlignmentResult {
    fn multiply_gcd(&mut self, gcd: u32) {
        self.penalty *= gcd;
    }
}

impl Penalty {
    fn new(mismatch: u32, gap_open: u32, gap_extend: u32) -> Self {
        Self {
            x: mismatch,
            o: gap_open,
            e: gap_extend,
        }
    }
    fn gcd_of_penalties(&self) -> u32 {
        integer::gcd(integer::gcd(self.x, self.o), self.e)
    }
    fn divide_by_gcd(&mut self, gcd: u32) {
        self.x /= gcd;
        self.o /= gcd;
        self.e /= gcd;
    }
}

impl Cutoff {
    fn new(minimum_aligned_length: u32, maximum_penalty_per_length: f32) -> Self {
        let maximum_penalty_per_scale = (maximum_penalty_per_length * PREC_SCALE as f32) as u32;
        Self::new_with_scaled_max_ppl(minimum_aligned_length, maximum_penalty_per_scale)
    }
    fn new_with_scaled_max_ppl(minimum_aligned_length: u32, maximum_penalty_per_scale: u32) -> Self {
        Self {
            minimum_aligned_length,
            maximum_penalty_per_scale,
        }
    }
    fn divide_by_gcd(&mut self, gcd: u32) {
        self.maximum_penalty_per_scale /= gcd;
    }
}

impl MinPenaltyForPattern {
    fn new(penalties: &Penalty) -> Self {
        let odd: u32;
        let even: u32;
        if penalties.x <= penalties.o + penalties.e {
            odd = penalties.x;
            if penalties.x * 2 <= penalties.o + (penalties.e * 2) {
                even = penalties.x;
            } else {
                even = penalties.o + (penalties.e * 2) - penalties.x;
            }
        } else {
            odd = penalties.o + penalties.e;
            even = penalties.e;
        }
        Self {
            odd,
            even
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gcd_calculation_for_penalties() {
        let mut penalties = Penalty::new(4, 6, 2);
        let gcd = penalties.gcd_of_penalties();
        assert_eq!(gcd, 2);
        penalties.divide_by_gcd(gcd);
        assert_eq!(penalties, Penalty::new(2, 3, 1));

        let mut penalties = Penalty::new(4, 5, 3);
        let gcd = penalties.gcd_of_penalties();
        assert_eq!(gcd, 1);
        penalties.divide_by_gcd(gcd);
        assert_eq!(penalties, Penalty::new(4, 5, 3));
    }

    #[allow(dead_code)]
    fn print_calculate_maximum_kmer() {
        let penalties = Penalty::new(4, 6, 2);
        let cutoff = Cutoff::new(50, 0.15);
        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let pattern_size = calculate_max_pattern_size(&cutoff, &min_penalty_for_pattern);
        println!("{}", pattern_size);
    }
}
