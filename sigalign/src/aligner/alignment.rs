use crate::{Result, error_msg};
use super::{ReferenceInterface, Sequence};
use super::{AlignmentResultsByRecordIndex, AlignmentResultsWithLabelByRecordIndex, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use super::{Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
use super::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};
use super::{Algorithm, SemiGlobalAlgorithm, LocalAlgorithm};
use super::{Reference, SequenceProvider, Labeling};
use super::FastaReader;
use super::{Aligner, WaveFrontHolder};
use super::raw_result_to_json;

impl Aligner {
    // Condition Checkers
    fn query_is_in_reference_bound(
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
    ) -> Result<()> {
        if reference.is_searchable(query) {
            Ok(())
        } else {
            error_msg!("Query string is not supported sequence type of reference.")
        }
    }
    fn multiply_gcd_to_alignment_results(
        &self,
        alignment_results_by_record_index: &mut AlignmentResultsByRecordIndex
    ) {
        alignment_results_by_record_index.multiply_gcd(self.gcd)
    }

    /// Perform semi-global alignment and return json result.
    pub fn semi_global_alignment<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.semi_global_alignment_unchecked(reference, query);
        let alignment_results = raw_result_to_json(alignment_results_raw)?;

        Ok(alignment_results)
    }
    /// Perform semi-global alignment and return raw result.
    pub fn semi_global_alignment_raw<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> Result<AlignmentResultsByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;

        Ok(self.semi_global_alignment_unchecked(reference, query))
    }
    /// Perform semi-global alignment and return json labeled result.
    pub fn semi_global_alignment_labeled<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_labeled_raw = self.semi_global_alignment_labeled_raw(reference, query)?;
        let alignment_results_labeled = raw_result_to_json(alignment_results_labeled_raw)?;

        Ok(alignment_results_labeled)
    }
    /// Perform semi-global alignment and return raw labeled result.
    pub fn semi_global_alignment_labeled_raw<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<AlignmentResultsWithLabelByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.semi_global_alignment_raw(reference, query)?;
        let alignment_results_labeled_raw = alignment_results_raw.to_labeled_results(reference);

        Ok(alignment_results_labeled_raw)
    }
    /// Perform semi-global alignment without checking query is supported sequence type.
    pub fn semi_global_alignment_unchecked<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence,
    ) -> AlignmentResultsByRecordIndex {
        self.wave_front_holder.allocate_more_space_if_needed(
            query.len(),
            &self.penalties,
            &self.cutoff,
        );

        let mut alignment_results_by_record = SemiGlobalAlgorithm::alignment(
            reference,
            query,
            self.kmer,
            &self.penalties,
            &self.cutoff,
            &self.min_penalty_for_pattern,
            &mut self.wave_front_holder.primary_wave_front,
            &mut self.wave_front_holder.secondary_wave_front,
        );
        self.multiply_gcd_to_alignment_results(&mut alignment_results_by_record);
        alignment_results_by_record
    }
    /// Perform local alignment and return json result.
    pub fn local_alignment<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.local_alignment_unchecked(reference, query);
        let alignment_results = raw_result_to_json(alignment_results_raw)?;

        Ok(alignment_results)
    }
    /// Perform local alignment and return raw result.
    pub fn local_alignment_raw<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> Result<AlignmentResultsByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;

        Ok(self.local_alignment_unchecked(reference, query))
    }
    /// Perform local alignment and return json labeled result.
    pub fn local_alignment_labeled<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<String> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_labeled_raw = self.local_alignment_labeled_raw(reference, query)?;
        let alignment_results_labeled = raw_result_to_json(alignment_results_labeled_raw)?;

        Ok(alignment_results_labeled)
    }
    /// Perform local alignment and return raw labeled result.
    pub fn local_alignment_labeled_raw<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        query: Sequence
    ) -> Result<AlignmentResultsWithLabelByRecordIndex> {
        Self::query_is_in_reference_bound(reference, query)?;
        let alignment_results_raw = self.local_alignment_raw(reference, query)?;
        let alignment_results_labeled_raw = alignment_results_raw.to_labeled_results(reference);

        Ok(alignment_results_labeled_raw)
    }
    /// Perform local alignment without checking query is supported sequence type.
    pub fn local_alignment_unchecked<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        query: Sequence
    ) -> AlignmentResultsByRecordIndex {
        self.wave_front_holder.allocate_more_space_if_needed(
            query.len(),
            &self.penalties,
            &self.cutoff,
        );

        let mut alignment_results_by_record = LocalAlgorithm::alignment(
            reference,
            query,
            self.kmer,
            &self.penalties,
            &self.cutoff,
            &self.min_penalty_for_pattern,
            &mut self.wave_front_holder.primary_wave_front,
            &mut self.wave_front_holder.secondary_wave_front,
        );
        self.multiply_gcd_to_alignment_results(&mut alignment_results_by_record);
        alignment_results_by_record
    }

    /// Perform semi-global alignment from fasta file and return json result
    /// * Sequences of unsupported type will not included in result.
    pub fn semi_global_alignment_from_fasta_file<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        fasta_file: &str,
    ) -> Result<String> {
        let raw_result = self.semi_global_alignment_raw_from_fasta_file(reference, fasta_file)?;
        let json_result = raw_result_to_json(raw_result)?;

        Ok(json_result)
    }
    /// Perform local alignment from fasta file and return json result
    /// * Sequences of unsupported type will not included in result.
    pub fn local_alignment_from_fasta_file<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        fasta_file: &str,
    ) -> Result<String> {
        let raw_result = self.local_alignment_raw_from_fasta_file(reference, fasta_file)?;
        let json_result = raw_result_to_json(raw_result)?;

        Ok(json_result)
    }
    /// Perform semi-global alignment from fasta file and return labeled json result
    /// * Sequences of unsupported type will not included in result.
    pub fn semi_global_alignment_labeled_from_fasta_file<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        fasta_file: &str,
    ) -> Result<String> {
        let raw_result = self.semi_global_alignment_labeled_raw_from_fasta_file(reference, fasta_file)?;
        let json_result = raw_result_to_json(raw_result)?;

        Ok(json_result)
    }
    /// Perform local alignment from fasta file and return labeled json result
    /// * Sequences of unsupported type will not included in result.
    pub fn local_alignment_labeled_from_fasta_file<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        fasta_file: &str,
    ) -> Result<String> {
        let raw_result = self.local_alignment_labeled_raw_from_fasta_file(reference, fasta_file)?;
        let json_result = raw_result_to_json(raw_result)?;

        Ok(json_result)
    }
    /// Perform semi-global alignment from fasta file and return raw result
    /// * Sequences of unsupported type will not included in result.
    pub fn semi_global_alignment_raw_from_fasta_file<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        fasta_file: &str,
    ) -> Result<Vec<(String, AlignmentResultsByRecordIndex)>> {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        
        let result: Vec<(String, AlignmentResultsByRecordIndex)> = fasta_reader.into_iter().filter_map(|(label, query)| {
            let record_result = self.semi_global_alignment_raw(reference, &query);
            match record_result {
                Ok(result) => {
                    Some((label, result))
                },
                Err(_) => {
                    None
                },
            }
        }).collect();

        Ok(result)
    }
    /// Perform local alignment from fasta file and return raw result
    /// * Sequences of unsupported type will not included in result.
    pub fn local_alignment_raw_from_fasta_file<S: SequenceProvider>(
        &mut self,
        reference: &mut Reference<S>,
        fasta_file: &str,
    ) -> Result<Vec<(String, AlignmentResultsByRecordIndex)>> {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        
        let result: Vec<(String, AlignmentResultsByRecordIndex)> = fasta_reader.into_iter().filter_map(|(label, query)| {
            let record_result = self.local_alignment_raw(reference, &query);
            match record_result {
                Ok(result) => {
                    Some((label, result))
                },
                Err(_) => {
                    None
                },
            }
        }).collect();

        Ok(result)
    }
    /// Perform semi-global alignment from fasta file and return labeled raw result
    /// * Sequences of unsupported type will not included in result.
    pub fn semi_global_alignment_labeled_raw_from_fasta_file<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        fasta_file: &str,
    ) -> Result<Vec<(String, AlignmentResultsWithLabelByRecordIndex)>> {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        
        let result: Vec<(String, AlignmentResultsWithLabelByRecordIndex)> = fasta_reader.into_iter().filter_map(|(label, query)| {
            let record_result = self.semi_global_alignment_labeled_raw(reference, &query);
            match record_result {
                Ok(result) => {
                    Some((label, result))
                },
                Err(_) => {
                    None
                },
            }
        }).collect();

        Ok(result)
    }
    /// Perform local alignment from fasta file and return labeled raw result
    /// * Sequences of unsupported type will not included in result.
    pub fn local_alignment_labeled_raw_from_fasta_file<SL: SequenceProvider + Labeling>(
        &mut self,
        reference: &mut Reference<SL>,
        fasta_file: &str,
    ) -> Result<Vec<(String, AlignmentResultsWithLabelByRecordIndex)>> {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        
        let result: Vec<(String, AlignmentResultsWithLabelByRecordIndex)> = fasta_reader.into_iter().filter_map(|(label, query)| {
            let record_result = self.local_alignment_labeled_raw(reference, &query);
            match record_result {
                Ok(result) => {
                    Some((label, result))
                },
                Err(_) => {
                    None
                },
            }
        }).collect();

        Ok(result)
    }
}

impl WaveFrontHolder {
    fn allocate_more_space_if_needed(
        &mut self,
        query_length: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        if self.allocated_query_length < query_length {
            // If length: (100..=199) -> allocate 200
            let to_allocate_query_length = ((query_length / Self::QUERY_LENGTH_UNIT) + 1) * Self::QUERY_LENGTH_UNIT;

            let max_score = Self::calculate_max_score_from_length(penalties, &cutoff, to_allocate_query_length);

            // TODO: Assign additional space from existing wave front
            let allocated_wave_front = WaveFront::new_allocated(penalties, max_score);

            *self = Self {
                allocated_query_length: to_allocate_query_length,
                primary_wave_front: allocated_wave_front.clone(),
                secondary_wave_front: allocated_wave_front,
            };
        }
    }
}
