use crate::{Result, error_msg};
use crate::core::{ReferenceInterface, Sequence};
pub use crate::core::{AlignmentResultsByRecordIndex, AlignmentResultsWithLabelByRecordIndex, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use crate::core::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
use crate::core::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};
use crate::algorithm::{Algorithm, SemiGlobalAlgorithm, LocalAlgorithm};
use crate::reference::{Reference, SequenceProvider, Labeling};
use crate::utils::FastaReader;

mod interpreter;
mod alignment;

use interpreter::raw_result_to_json;

use num::integer;

/** `Aligner` to perform alignment process.

The `Aligner` is generated with **three** penalty values and **two** similarity cutoff values. In `sigalign`, only these five values affect the alignment result. There are two alignment algorithms, semi-global alignment and local alignment.

# (1) Parameters to generate `Aligner`
1. Mismatch penalty
    - Penalty value for mismatch (substitution).
2. Gap open penalty
    - Penalty value for gap-open
3. Gap extend penalty
    - Penalty value for gap-extend
4. Minimum aligned length
    - The length of alignment have to be greater than or equal to this value.
5. Maximum penalty per length
    - The penalty per length (penalty / length) of alignment have to be less than or equal to this value.
    - This value is also called dissimilarity or normalized penalty.

# (2) Alignment algorithms
1. Semi-global alignment
    - Return the results of alignment that either the target or query sequence extended to the end from the left and right sides of the sequence respectively.
2. Local alignment
    - This algorithm return the longest alignment among the alignment satisfying the similarity cutoff is output as a result.

The multiple results with different alignment positions can be returned in both algorithms.
*/
#[derive(Debug)]
pub struct Aligner {
    penalties: Penalties,
    cutoff: Cutoff,
    min_penalty_for_pattern: MinPenaltyForPattern,
    gcd: usize,
    kmer: usize,
    wave_front_holder: WaveFrontHolder,
}

struct WaveFrontHolder {
    allocated_query_length: usize,
    primary_wave_front: WaveFront,
    secondary_wave_front: WaveFront,
}

impl Aligner {
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

        Ok(aligner)
    }
    fn new_with_penalties_and_cutoff(mut penalties: Penalties, mut cutoff: Cutoff) -> Self {
        let gcd = penalties.gcd_of_penalties();
        penalties.divide_by_gcd(gcd);
        cutoff.divide_by_gcd(gcd);

        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_kmer = Self::max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let wave_front_holder = WaveFrontHolder::new(&penalties, &cutoff);
        
        Self {
            penalties,
            cutoff,
            min_penalty_for_pattern,
            gcd,
            kmer: max_kmer,
            wave_front_holder,
        }
    }
    fn max_kmer_satisfying_cutoff(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
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

            let kmer = max_penalty.min(upper_bound);

            if kmer >= lower_bound {
                return kmer as usize
            }
            n += 1;
        }
    }
    /// Get penalties
    pub fn get_penalties(&self) -> [usize; 3] {
        [
            self.penalties.x * self.gcd,
            self.penalties.o * self.gcd,
            self.penalties.e * self.gcd,
        ]
    }
    /// Get similarity cutoff
    pub fn get_similarity_cutoff(&self) -> (usize, f32) {
        (
            self.cutoff.minimum_aligned_length,
            (self.cutoff.maximum_penalty_per_scale * self.gcd) as f32 / PRECISION_SCALE as f32,
        )
    }
    /// Get size of pattern
    pub fn get_pattern_size(&self) -> usize {
        self.kmer
    }
}

impl WaveFrontHolder {
    const QUERY_LENGTH_UNIT: usize = 100;

    fn new(
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> Self {
        let to_allocate_query_length = Self::QUERY_LENGTH_UNIT;
        let max_score = Self::calculate_max_score_from_length(penalties, cutoff, to_allocate_query_length);

        let allocated_wave_front = WaveFront::new_allocated(penalties, max_score);

        Self {
            allocated_query_length: to_allocate_query_length,
            primary_wave_front: allocated_wave_front.clone(),
            secondary_wave_front: allocated_wave_front,
        }
    }
    fn calculate_max_score_from_length(
        penalties: &Penalties,
        cutoff: &Cutoff,
        query_length: usize,
    ) -> usize {
        (
            cutoff.maximum_penalty_per_scale * (
                penalties.e * query_length - penalties.o
            )
        ) / (
            PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) + 1
    }
}

use std::fmt;
impl fmt::Debug for WaveFrontHolder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WaveFrontHolder")
         .field("allocated_query_length", &self.allocated_query_length)
         .finish()
    }
}

impl AlignmentResultsByRecordIndex {
    fn multiply_gcd(&mut self, gcd: usize) {
        self.0.iter_mut().for_each(|(_, alignment_results)| {
            alignment_results.iter_mut().for_each(|alignment_result| {
                alignment_result.multiply_gcd(gcd);
            })
        })
    }
}

impl AlignmentResult {
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
        let penalty_per_scale = (maximum_penalty_per_length * PRECISION_SCALE as f32) as usize;
        Self {
            minimum_aligned_length,
            maximum_penalty_per_scale: penalty_per_scale,
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
        let kmer = Aligner::max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);
        println!("{}", kmer);
    }
}
