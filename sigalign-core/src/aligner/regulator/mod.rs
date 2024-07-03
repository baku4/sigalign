use crate::core::regulators::{
    Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
    calculate_max_pattern_size,
};
use crate::results::{
    QueryAlignment, Alignment, TargetAlignment,
};
use thiserror::Error;
use num::integer::gcd;

/// Error to define the regulator.
#[derive(Error, Debug)]
pub enum RegulatorError {
    #[error("Gap extend penalty only allow positive integer.")]
    InvalidGapExtendPenalty,
    #[error("Maximum penalty per length only allow positive value.")]
    InvalidMaxPenaltyPerLength,
}

/// Definition for the alignment results.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlignmentRegulator {
    pub(super) penalties: Penalty,
    pub(super) cutoff: Cutoff,
    pub(super) min_penalty_for_pattern: MinPenaltyForPattern,
    pub(super) gcd_for_compression: u32,
    pub(super) pattern_size: u32,
}

impl AlignmentRegulator {
    /// Generate new aligner.
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_alignment_length: u32,
        maximum_penalty_per_alignment_length: f32,
    ) -> Result<Self, RegulatorError> {
        if gap_extend_penalty == 0 {
            return Err(RegulatorError::InvalidGapExtendPenalty);
        } else if maximum_penalty_per_alignment_length <= 0.0 {
            return Err(RegulatorError::InvalidMaxPenaltyPerLength);
        }

        let penalties = Penalty::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty);
        let cutoff = Cutoff::new(minimum_alignment_length, maximum_penalty_per_alignment_length);
        let aligner = Self::new_with_gcd_compressed_from_penalties_and_cutoff(penalties, cutoff);
        
        Ok(aligner)
    }
    fn new_with_gcd_compressed_from_penalties_and_cutoff(mut penalties: Penalty, mut cutoff: Cutoff) -> Self {
        let gcd = penalties.gcd_of_penalties();
        penalties.divide_by_gcd(gcd);
        cutoff.divide_by_gcd(gcd);

        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_pattern_size = calculate_max_pattern_size(
            &cutoff,
            &min_penalty_for_pattern,
            penalties.e,
        );
        
        Self {
            penalties,
            cutoff,
            min_penalty_for_pattern,
            gcd_for_compression: gcd,
            pattern_size: max_pattern_size,
        }
    }
    pub(super) fn decompress_result_with_gcd(&self, alignment_result: &mut QueryAlignment) {
        if self.gcd_for_compression != 1 {
            alignment_result.multiply_gcd(self.gcd_for_compression);
        }
    }
    /// Get mismatch penalty
    pub fn get_mismatch_penalty(&self) -> u32 {
        self.penalties.x * self.gcd_for_compression
    }
    /// Get gap-open penalty
    pub fn get_gap_open_penalty(&self) -> u32 {
        self.penalties.o * self.gcd_for_compression
    }
    /// Get gap-extend penalty
    pub fn get_gap_extend_penalty(&self) -> u32 {
        self.penalties.e * self.gcd_for_compression
    }
    /// Get minimum length
    pub fn get_minimum_length(&self) -> u32 {
        self.cutoff.minimum_length
    }
    /// Get maximum penalty per length
    pub fn get_maximum_penalty_per_length(&self) -> f32 {
        (self.cutoff.maximum_scaled_penalty_per_length * self.gcd_for_compression) as f32 / PREC_SCALE as f32
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> u32 {
        self.pattern_size
    }
}

impl QueryAlignment {
    fn multiply_gcd(&mut self, gcd: u32) {
        self.0.iter_mut().for_each(|target_alignment_result| {
            target_alignment_result.multiply_gcd(gcd);
        })
    }
}

impl TargetAlignment {
    #[inline]
    fn multiply_gcd(&mut self, gcd: u32) {
        self.alignments.iter_mut().for_each(|alignment_result| {
            alignment_result.multiply_gcd(gcd);
        })
    }
}

impl Alignment {
    #[inline]
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
        gcd(gcd(self.x, self.o), self.e)
    }
    fn divide_by_gcd(&mut self, gcd: u32) {
        self.x /= gcd;
        self.o /= gcd;
        self.e /= gcd;
    }
}

impl Cutoff {
    fn new(minimum_length: u32, maximum_penalty_per_length: f32) -> Self {
        let maximum_penalty_per_scale = (maximum_penalty_per_length * PREC_SCALE as f32) as u32;
        Self::new_with_scaled_max_ppl(minimum_length, maximum_penalty_per_scale)
    }
    fn new_with_scaled_max_ppl(minimum_length: u32, maximum_penalty_per_scale: u32) -> Self {
        Self {
            minimum_length,
            maximum_scaled_penalty_per_length: maximum_penalty_per_scale,
        }
    }
    fn divide_by_gcd(&mut self, gcd: u32) {
        self.maximum_scaled_penalty_per_length /= gcd;
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
        let pattern_size = calculate_max_pattern_size(
            &cutoff,
            &min_penalty_for_pattern,
            penalties.e,
        );
        println!("{}", pattern_size);
    }
}
