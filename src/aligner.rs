use num::integer;

pub struct Aligner {
    penalties: Penalties,
    cutoff: Cutoff,
    min_penalty_for_pattern: MinPenaltyForPattern,
    kmer: usize,
}

impl Aligner {
    fn new(penalties: Penalties, cutoff: Cutoff) -> Self {
        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_kmer = Self::max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);
        Self {
            penalties,
            cutoff,
            min_penalty_for_pattern,
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
    // fn align(reference: &dyn Reference, search_range: &SearchRange, query: Query) -> AlignmentResult;
}

pub struct Penalties {
    x: usize,
    o: usize,
    e: usize,
    gcd: usize,
}

impl Penalties {
    pub fn new(mismatch: usize, gap_open: usize, gap_extend: usize) -> Self {
        let gcd = integer::gcd(integer::gcd(mismatch, gap_open), gap_extend);

        Self {
            x: mismatch / gcd,
            o: gap_open / gcd,
            e: gap_extend / gcd,
            gcd,
        }
    }
    fn mismatch(&self) -> usize {
        self.x * self.gcd
    }
    fn gap_open(&self) -> usize {
        self.o * self.gcd
    }
    fn gap_extend(&self) -> usize {
        self.e * self.gcd
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
}

pub struct MinPenaltyForPattern {
    odd: usize,
    even: usize,
}

impl MinPenaltyForPattern {
    pub fn new(penalties: &Penalties) -> Self {
        let mismatch = penalties.mismatch();
        let gap_open = penalties.gap_open();
        let gap_extend = penalties.gap_extend();

        let odd: usize;
        let even: usize;
        if mismatch <= gap_open + gap_extend {
            odd = mismatch;
            if mismatch * 2 <= gap_open + (gap_extend * 2) {
                even = mismatch;
            } else {
                even = gap_open + (gap_extend * 2) - mismatch;
            }
        } else {
            odd = gap_open + gap_extend;
            even = gap_extend;
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
    fn print_calculate_maximum_kmer() {
        let penalties = Penalties::new(4, 6, 2);
        let cutoff = Cutoff::new(100, 0.1);
        let mpfp = MinPenaltyForPattern::new(&penalties);
        let kmer = Aligner::max_kmer_satisfying_cutoff(&cutoff, &mpfp);
        println!("{}", kmer);
    }
}