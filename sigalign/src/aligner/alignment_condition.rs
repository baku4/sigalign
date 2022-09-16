use super::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, PatternLocation,
    Reference, SequenceProvider,
};
use num::integer;

const MINIMUM_PATTERN_SIZE: usize = 4;

pub fn calculate_max_pattern_size(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
    let mut n = 1;
    loop { // TODO: Optimize
        let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
        let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
        let max_penalty = (
            (
                (
                    (PRECISION_SCALE * n * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even))
                )
                + 4 * cutoff.maximum_penalty_per_scale
            ) as f32 / (2 * (n+1) * cutoff.maximum_penalty_per_scale) as f32
        ).ceil() - 2_f32;

        let pattern_size = max_penalty.min(upper_bound);

        if pattern_size >= lower_bound {
            return pattern_size as usize
        }
        n += 1;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct AlignmentCondition {
    pub penalties: Penalties,
    pub cutoff: Cutoff,
    pub min_penalty_for_pattern: MinPenaltyForPattern,
    pub gcd_for_compression: usize,
    pub pattern_size: usize,
}

impl AlignmentCondition {
    /// Generate new aligner.
    pub fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Self> {
        if gap_extend_penalty == 0 {
            error_msg!("Gap extend penalty only allow positive integer.");
        } else if maximum_penalty_per_length <= 0.0 {
            error_msg!("Maximum penalty per length only allow positive value.");
        }

        let penalties = Penalties::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty);
        let cutoff = Cutoff::new(minimum_aligned_length, maximum_penalty_per_length);

        let aligner = Self::new_with_penalties_and_cutoff(penalties, cutoff);

        let pattern_size = &aligner.pattern_size;
        if *pattern_size < MINIMUM_PATTERN_SIZE {
            error_msg!("Auto calculated pattern size({}) should reach at least {}", pattern_size, MINIMUM_PATTERN_SIZE);
        }

        Ok(aligner)
    }
    fn new_with_penalties_and_cutoff(mut penalties: Penalties, mut cutoff: Cutoff) -> Self {
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
    pub fn get_penalties(&self) -> [usize; 3] {
        [
            self.penalties.x * self.gcd_for_compression,
            self.penalties.o * self.gcd_for_compression,
            self.penalties.e * self.gcd_for_compression,
        ]
    }
    /// Get similarity cutoff
    pub fn get_similarity_cutoff(&self) -> (usize, f32) {
        (
            self.cutoff.minimum_aligned_length,
            (self.cutoff.maximum_penalty_per_scale * self.gcd_for_compression) as f32 / PRECISION_SCALE as f32,
        )
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> usize {
        self.pattern_size
    }
}

impl AlignmentResult {
    fn multiply_gcd(&mut self, gcd: usize) {
        self.0.iter_mut().for_each(|record_alignment_result| {
            record_alignment_result.alignments.iter_mut().for_each(|alignment_result| {
                alignment_result.multiply_gcd(gcd);
            })
        })
    }
}

impl AnchorAlignmentResult {
    fn multiply_gcd(&mut self, gcd: usize) {
        self.penalty *= gcd;
    }
}

impl Penalties {
    fn new(mismatch: usize, gap_open: usize, gap_extend: usize) -> Self {
        Self {
            x: mismatch,
            o: gap_open,
            e: gap_extend,
        }
    }
    fn gcd_of_penalties(&self) -> usize {
        integer::gcd(integer::gcd(self.x, self.o), self.e)
    }
    fn divide_by_gcd(&mut self, gcd: usize) {
        self.x /= gcd;
        self.o /= gcd;
        self.e /= gcd;
    }
}

impl Cutoff {
    fn new(minimum_aligned_length: usize, maximum_penalty_per_length: f32) -> Self {
        let maximum_penalty_per_scale = (maximum_penalty_per_length * PRECISION_SCALE as f32) as usize;
        Self::new_with_scaled_max_ppl(minimum_aligned_length, maximum_penalty_per_scale)
    }
    fn new_with_scaled_max_ppl(minimum_aligned_length: usize, maximum_penalty_per_scale: usize) -> Self {
        Self {
            minimum_aligned_length,
            maximum_penalty_per_scale,
        }
    }
    fn divide_by_gcd(&mut self, gcd: usize) {
        self.maximum_penalty_per_scale /= gcd;
    }
}

impl MinPenaltyForPattern {
    fn new(penalties: &Penalties) -> Self {
        let odd: usize;
        let even: usize;
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

    fn test_gcd_calculation_for_penalties() {
        let mut penalties = Penalties::new(4, 6, 2);
        let gcd = penalties.gcd_of_penalties();
        assert_eq!(gcd, 2);
        penalties.divide_by_gcd(gcd);
        assert_eq!(penalties, Penalties::new(2, 3, 1));

        let mut penalties = Penalties::new(4, 5, 3);
        let gcd = penalties.gcd_of_penalties();
        assert_eq!(gcd, 1);
        penalties.divide_by_gcd(gcd);
        assert_eq!(penalties, Penalties::new(4, 5, 3));
    }

    #[test]
    fn print_calculate_maximum_kmer() {
        let penalties = Penalties::new(4, 6, 2);
        let cutoff = Cutoff::new(50, 0.15);
        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let pattern_size = calculate_max_pattern_size(&cutoff, &min_penalty_for_pattern);
        println!("{}", pattern_size);
    }
}
