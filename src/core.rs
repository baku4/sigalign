use std::collections::HashMap;

mod anchoring;
mod extending;
mod evaluating;

pub use extending::{DropoffWaveFront, WaveFrontScore, Components, Component};
pub use extending::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};
pub use evaluating::AlignmentHashSet;


// CONDITIONS


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalties {
    pub x: usize,
    pub o: usize,
    pub e: usize,
}

pub struct Cutoff {
    pub minimum_aligned_length: usize,
    pub penalty_per_length: f32,
}

pub struct MinPenaltyForPattern {
    pub odd: usize,
    pub even: usize,
}


// TEXT


pub type Sequence<'a> = &'a [u8];

pub trait Reference {
    fn locate(&self, pattern: Sequence, kmer: usize) -> Vec<PatternLocation>;
    fn sequence_of_record(&self, record_index: usize) -> Sequence;
}

pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}


// RESULTS


pub type AlignmentResultsByRecord = HashMap<usize, Vec<AlignmentResult>>;

#[derive(Debug)]
pub struct AlignmentResult {
    pub dissimilarity: f32,
    pub penalty: usize,
    pub length: usize,
    pub position: AlignmentPosition,
    pub operations: Vec<AlignmentOperation>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct AlignmentPosition {
    pub record: (usize, usize),
    pub query: (usize, usize),
}

#[derive(Debug, Clone)]
pub struct AlignmentOperation {
    pub alignment_type: AlignmentType,
    pub count: u32,
}

#[derive(Debug, Clone)]
pub enum AlignmentType {
    Match,
    Subst,
    Insertion,
    Deletion,
}


// ANCHOR


pub trait Anchors {
    // fn from_preset(anchor_preset: AnchorsPreset) -> Self;
}


/*

#[cfg(test)]
mod tests {
    use super::*;

    use crate::reference::TestReference;

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

    #[test]
    fn print_results_of_semi_global_alignment() {
        let test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff { minimum_aligned_length: 30, penalty_per_length: 0.3 };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let alignment_results = semi_global_alignment(
            &test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern
        );

        println!("alignment_results:\n{:#?}", alignment_results);
    }
    #[test]
    fn print_results_of_local_alignment() {
        let test_reference = TestReference::new();

        let query = b"CGGATGCTCCGGCAGCCGACAGAACGAAGGATCTTGCCGGAAAATGAACTTCTGTTATTATTTTTGTGATTCA";

        let penalties = Penalties {x: 4, o: 5, e: 2};
        let cutoff = Cutoff { minimum_aligned_length: 30, penalty_per_length: 0.3 };
        let min_penalty_for_pattern = MinPenaltyForPattern { odd: 4, even: 3 };

        let kmer = max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);

        let alignment_results = local_alignment(
            &test_reference,
            query,
            kmer,
            &penalties,
            &cutoff,
            &min_penalty_for_pattern
        );

        println!("alignment_results:\n{:#?}", alignment_results);
    }
}

*/