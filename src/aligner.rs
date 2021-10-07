use bio::io::fastq::Reader;
use num::integer;

mod alignment_result;

use crate::{Result, error_msg};
use alignment_result::AlignmentResult;

pub struct Aligner {
    penalties: Penalties,
    cutoff: Cutoff,
    gcd: usize,
    kmer: usize,
}

impl Aligner {
    pub fn new(mut penalties: Penalties, mut cutoff: Cutoff) -> Self {
        let gcd = penalties.gcd_of_penalties();
        penalties.divide_by_gcd(gcd);
        cutoff.divide_by_gcd(gcd);

        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_kmer = Self::max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);
        
        Self {
            penalties,
            cutoff,
            gcd,
            kmer: max_kmer,
        }
    }
    fn max_kmer_satisfying_cutoff(cutoff: &Cutoff, min_penalty_for_pattern: &MinPenaltyForPattern) -> usize {
        let mut n = 1;
        loop {
            let upper_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n)  as f32 - 2_f32).ceil();
            let lower_bound = ((cutoff.minimum_aligned_length + 4)  as f32 / (2*n + 2)  as f32 - 2_f32).ceil();
            let max_penalty = (
                ((n*(min_penalty_for_pattern.odd + min_penalty_for_pattern.even)) as f32
                + 4_f32*cutoff.penalty_per_length) /
                (2_f32*cutoff.penalty_per_length*(n+1) as f32)
                - 2_f32
            ).ceil();

            let kmer = max_penalty.min(upper_bound);
            #[cfg(test)]
            println!("#n {}\nu {}\nl {}\nmp {}\nk {}", n, upper_bound, lower_bound, max_penalty, kmer);

            if kmer >= lower_bound {
                return kmer as usize
            }
            n += 1;
        }
    }

    pub fn semi_global(&self, reference: &dyn Reference, query: Query) { //-> Result<AlignmentResult> {
        
    }
}

#[derive(Debug, PartialEq)]
pub struct Penalties {
    x: usize,
    o: usize,
    e: usize,
}

impl Penalties {
    pub fn new(mismatch: usize, gap_open: usize, gap_extend: usize) -> Self {
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

pub struct Cutoff {
    minimum_aligned_length: usize,
    penalty_per_length: f32,
}

impl Cutoff {
    pub fn new(minimum_aligned_length: usize, penalty_per_length: f32) -> Self {
        Self {
            minimum_aligned_length,
            penalty_per_length,
        }
    }
    fn divide_by_gcd(&mut self, gcd: usize) {
        self.penalty_per_length /= gcd as f32;
    }
}

pub struct MinPenaltyForPattern {
    odd: usize,
    even: usize,
}

impl MinPenaltyForPattern {
    pub fn new(penalties: &Penalties) -> Self {
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

pub trait Reference {
    
}

struct SearchRange {
    sorted_vector: Vec<usize>,
}

type Query<'a> = &'a [u8];


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
        let mpfp = MinPenaltyForPattern::new(&penalties);
        let kmer = Aligner::max_kmer_satisfying_cutoff(&cutoff, &mpfp);
        println!("{}", kmer);
    }
}