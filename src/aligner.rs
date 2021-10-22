use std::collections::HashMap;

use crate::{Result, error_msg};
use crate::core::{ReferenceInterface, Sequence};
use crate::core::{AlignmentResultsByRecordIndex, AlignmentResultsByRecordLabel, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use crate::core::{Penalties, Cutoff, MinPenaltyForPattern};
use crate::algorithm::{Algorithm, SemiGlobalAlgorithm, LocalAlgorithm};
use crate::reference::{Reference, SequenceProvider, Labeling};

mod interpreter;

use num::integer;

pub struct Aligner {
    penalties: Penalties,
    cutoff: Cutoff,
    min_penalty_for_pattern: MinPenaltyForPattern,
    gcd: usize,
    kmer: usize,
}

impl Aligner {
    pub fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        penalty_per_length: f32,
    ) -> Result<Self> {
        if gap_extend_penalty == 0 {
            error_msg!(""); //TODO: Err msg
        } else if penalty_per_length <= 0.0 {
            error_msg!(""); //TODO: Err msg
        }

        let penalties = Penalties::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty);
        let cutoff = Cutoff::new(minimum_aligned_length, penalty_per_length);

        let aligner = Self::new_with_penalties_and_cutoff(penalties, cutoff);

        Ok(aligner)
    }
    pub fn new_with_penalties_and_cutoff(mut penalties: Penalties, mut cutoff: Cutoff) -> Self {
        let gcd = penalties.gcd_of_penalties();
        penalties.divide_by_gcd(gcd);
        cutoff.divide_by_gcd(gcd);

        let min_penalty_for_pattern = MinPenaltyForPattern::new(&penalties);
        let max_kmer = Self::max_kmer_satisfying_cutoff(&cutoff, &min_penalty_for_pattern);
        
        Self {
            penalties,
            cutoff,
            min_penalty_for_pattern,
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
    pub fn semi_global_alignment<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.semi_global_alignment_unchecked(reference, query);
        let alignment_results = alignment_results_raw.to_json()?;

        Ok(alignment_results)
    }
    pub fn semi_global_alignment_raw<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> Result<AlignmentResultsByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;

        Ok(self.semi_global_alignment_unchecked(reference, query))
    }
    pub fn semi_global_alignment_labeled<SL: SequenceProvider + Labeling>(
        &self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_labeled_raw = self.semi_global_alignment_labeled_raw(reference, query)?;
        let alignment_results_labeled = alignment_results_labeled_raw.to_json()?;

        Ok(alignment_results_labeled)
    }
    pub fn semi_global_alignment_labeled_raw<SL: SequenceProvider + Labeling>(
        &self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<AlignmentResultsByRecordLabel> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.semi_global_alignment_raw(reference, query)?;
        let alignment_results_labeled_raw = alignment_results_raw.to_labeled_results(reference);

        Ok(alignment_results_labeled_raw)
    }
    pub fn semi_global_alignment_unchecked<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> AlignmentResultsByRecordIndex {
        let mut alignment_results_by_record = SemiGlobalAlgorithm::alignment(reference, query, self.kmer, &self.penalties, &self.cutoff, &self.min_penalty_for_pattern);
        self.multiply_gcd_to_alignment_results(&mut alignment_results_by_record);
        alignment_results_by_record
    }
    pub fn local_alignment<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.local_alignment_unchecked(reference, query);
        let alignment_results = alignment_results_raw.to_json()?;

        Ok(alignment_results)
    }
    pub fn local_alignment_raw<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> Result<AlignmentResultsByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;

        Ok(self.local_alignment_unchecked(reference, query))
    }
    pub fn local_alignment_labeled<SL: SequenceProvider + Labeling>(
        &self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_labeled_raw = self.local_alignment_labeled_raw(reference, query)?;
        let alignment_results_labeled = alignment_results_labeled_raw.to_json()?;

        Ok(alignment_results_labeled)
    }
    pub fn local_alignment_labeled_raw<SL: SequenceProvider + Labeling>(
        &self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<AlignmentResultsByRecordLabel> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.local_alignment_raw(reference, query)?;
        let alignment_results_labeled_raw = alignment_results_raw.to_labeled_results(reference);

        Ok(alignment_results_labeled_raw)
    }
    pub fn local_alignment_unchecked<S: SequenceProvider>(
        &self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> AlignmentResultsByRecordIndex {
        let mut alignment_results_by_record = LocalAlgorithm::alignment(reference, query, self.kmer, &self.penalties, &self.cutoff, &self.min_penalty_for_pattern);
        self.multiply_gcd_to_alignment_results(&mut alignment_results_by_record);
        alignment_results_by_record
    }
    fn query_is_in_reference_bound(
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
    ) -> Result<()> {
        if reference.is_searchable(query) {
            Ok(())
        } else {
            error_msg!("Query string is not included in the sequence type of reference.")
        }
    }
    fn multiply_gcd_to_alignment_results(
        &self,
        alignment_results_by_record_index: &mut AlignmentResultsByRecordIndex
    ) {
        alignment_results_by_record_index.multiply_gcd(self.gcd)
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
        self.dissimilarity *= gcd as f32;
        self.penalty *= gcd;
    }
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
