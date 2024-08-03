use crate::core::regulators::{
    Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
};
use crate::results::{
    AlignmentResult, AnchorAlignmentResult,
};
use thiserror::Error;
use num::integer::{div_ceil, gcd};

#[derive(Error, Debug)]
pub enum RegulatorError {
    #[error("Cutoff is too low to detect the pattern.")]
    LowCutoff,
    #[error("Gap extend penalty only allow positive integer.")]
    InvalidGapExtendPenalty,
    #[error("Maximum penalty per length only allow positive value.")]
    InvalidMPpL,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlignmentRegulator {
    pub penalties: Penalty,
    pub cutoff: Cutoff,
    pub min_penalty_for_pattern: MinPenaltyForPattern,
    pub gcd_for_compression: u32,
    pub pattern_size: u32,
}

const MINIMUM_PATTERN_SIZE: u32 = 4;
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
            return Err(RegulatorError::InvalidGapExtendPenalty);
        } else if maximum_penalty_per_length <= 0.0 {
            return Err(RegulatorError::InvalidMPpL);
        }

        let penalties = Penalty::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty);
        let cutoff = Cutoff::new(minimum_aligned_length, maximum_penalty_per_length);

        let aligner = Self::new_with_penalties_and_cutoff(penalties, cutoff);

        let pattern_size = &aligner.pattern_size;
        if *pattern_size < MINIMUM_PATTERN_SIZE {
            return Err(RegulatorError::LowCutoff);
        }

        Ok(aligner)
    }
    fn new_with_penalties_and_cutoff(mut penalties: Penalty, mut cutoff: Cutoff) -> Self {
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
    pub fn result_of_uncompressed_penalty(&self, mut reference_alignment_result: AlignmentResult) -> AlignmentResult {
        if self.gcd_for_compression != 1 {
            reference_alignment_result.multiply_gcd(self.gcd_for_compression);
        }
        
        reference_alignment_result
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
    /// Get minimum aligned length
    pub fn get_minimum_aligned_length(&self) -> u32 {
        self.cutoff.minimum_aligned_length
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
        gcd(gcd(self.x, self.o), self.e)
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
            maximum_scaled_penalty_per_length: maximum_penalty_per_scale,
        }
    }
    fn divide_by_gcd(&mut self, gcd: u32) {
        self.maximum_scaled_penalty_per_length /= gcd;
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

// For pattern size calculation
#[inline(always)]
pub fn calculate_max_pattern_size(
    cutoff: &Cutoff,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    gap_extend_penalty: u32,
) -> u32 {
    // Initialize
    let mut lower_bound = get_lower_bound_of_n(
        0,
        cutoff.minimum_aligned_length,
        gap_extend_penalty,
        min_penalty_for_pattern.odd,
    );
    let mut upper_bound;
    let mut n = 0;
    let mut k = 0;
    let mut k1;
    let mut k2 = 0;
    
    while k < lower_bound {
        n += 1;
        upper_bound = lower_bound - 1;
        lower_bound = get_lower_bound_of_n(
            n,
            cutoff.minimum_aligned_length,
            gap_extend_penalty,
            min_penalty_for_pattern.odd,
        );

        if n % 2 == 1 { // If n is odd
            let m = (n+1) / 2;
            k1 = max_k_satisfying_maxp_for_odd(
                m,
                gap_extend_penalty,
                min_penalty_for_pattern.odd,
                min_penalty_for_pattern.even,
                cutoff.maximum_scaled_penalty_per_length,
            );
            k2 = max_k_satisfying_maxp_for_even(
                m,
                gap_extend_penalty,
                min_penalty_for_pattern.odd,
                min_penalty_for_pattern.even,
                cutoff.maximum_scaled_penalty_per_length,
            );
            k = upper_bound.min(k1).min(k2);
        } else {
            k = upper_bound.min(k2);
        }
    }
    k
}
#[inline(always)]
fn get_lower_bound_of_n(
    n: u32, // Number of patterns
    minl: u32, // Minimum length
    pe: u32, // Gap-extend penalty
    p1: u32, // Penalty for odd pattern
) -> u32 {
    let numerator = pe*(minl+4) - p1;
    let denominator = pe*(n+2);
    div_ceil(numerator, denominator) - 1
}
#[inline(always)]
fn max_k_satisfying_maxp_for_odd(
    m: u32,
    pe: u32,
    p1: u32,
    p2: u32,
    scaled_maxp: u32,
) -> u32 {
    let numerator = PREC_SCALE*pe*(m*p1+m*p2-p2) + 4*scaled_maxp*pe - scaled_maxp*p1;
    let denominator = scaled_maxp*pe*(2*m+1);
    div_ceil(numerator, denominator) - 2
}
#[inline(always)]
fn max_k_satisfying_maxp_for_even(
    m: u32,
    pe: u32,
    p1: u32,
    p2: u32,
    scaled_maxp: u32,
) -> u32 {
    let numerator = PREC_SCALE*m*pe*(p1+p2) + 4*scaled_maxp*pe - scaled_maxp*p1;
    let denominator = scaled_maxp*pe*(2*m+2);
    div_ceil(numerator, denominator) - 2
}