use super::*;
use crate::core::*;
use crate::reference::*;
use crate::reference::sequence_provider::*;
use crate::algorithm::*;
use crate::aligner::*;

mod sample_data;
use sample_data::*;

mod standard_aligner;
use standard_aligner::*; 

mod fasta_reader;
use fasta_reader::*;

use std::collections::HashSet;
use std::time::{Duration, Instant};

#[test]
fn test_results_of_nucleotide_only_for_semi_global_with_in_memory_provider() {
    println!("Use In-Memory Provider");

    let fasta_file_for_reference = NUCLEOTIDE_ONLY_FA_PATH_1;

    let sequence_provider = {
        let start_time = Instant::now();
        let sequence_provider = InMemoryProvider::from_fasta_file(fasta_file_for_reference).unwrap();
        let duration = start_time.elapsed().as_millis();
        println!("Generate sequence provider: {:?}", duration);

        sequence_provider
    };
    
    print_results_of_nucleotide_only_for_semi_global(
        fasta_file_for_reference,
        sequence_provider,
    );
}

fn print_results_of_nucleotide_only_for_semi_global<S: SequenceProvider>(
    fasta_file_for_reference: &str,
    sequence_provider: S,
) {
    let mismatch_penalty = 4;
    let gap_open_penalty = 6;
    let gap_extend_penalty = 2;
    let minimum_aligned_length = 100;
    let penalty_per_length = 0.1;

    let kmer_size_for_lookup_table = 4;

    // For this crate
    let aligner = Aligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

    println!("Aligner: {:?}", aligner);

    let sequence_type = SequenceType::nucleotide_only();

    let mut reference = {
        let start_time = Instant::now();

        let lt_fm_index_config = LtFmIndexConfig::new()
            .change_kmer_size_for_lookup_table(kmer_size_for_lookup_table);

        let reference = Reference::new(sequence_type.clone(), lt_fm_index_config, sequence_provider).unwrap();
        
        let duration = start_time.elapsed().as_millis();
        println!("Generate reference: {:?}", duration);

        reference
    };

    // For comparison
    let standard_aligner = StandardAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length);

    println!("Standard Aligner: {:?}", standard_aligner);

    let standard_reference = {
        let start_time = Instant::now();

        let standard_reference = StandardReference::new_from_fasta(
            sequence_type,
            fasta_file_for_reference
        );

        let duration = start_time.elapsed().as_millis();
        println!("Generate standard reference: {:?}", duration);

        standard_reference
    };  

    let query_fasta_records = FastaReader::from_file_path(NUCLEOTIDE_ONLY_FA_PATH_2).unwrap();

    for (query_record_index, (label, query)) in query_fasta_records.into_iter().enumerate() {
        println!("{:?} - {:?}", query_record_index, label);

        let start_time_1 = Instant::now();
        let result_of_this_crate = aligner.semi_global_alignment_unchecked(
            &mut reference,
            &query
        );
        let duration_1 = start_time_1.elapsed().as_millis();
    
        let start_time_2 = Instant::now();
        let result_of_standard = standard_aligner.semi_global_alignment_raw(
            &standard_reference,
            &query
        );
        let duration_2 = start_time_2.elapsed().as_millis();

        println!("Time elapsed: {}, {}", duration_1, duration_2);

        print_alignment_results_by_index_are_same(
            &result_of_this_crate,
            &result_of_standard,
        );
    }
}

#[test]
fn test_results_of_nucleotide_only_for_local_with_in_memory_provider() {
    println!("Use In-Memory Provider");

    let fasta_file_for_reference = NUCLEOTIDE_ONLY_FA_PATH_1;

    let sequence_provider = {
        let start_time = Instant::now();
        let sequence_provider = InMemoryProvider::from_fasta_file(fasta_file_for_reference).unwrap();
        let duration = start_time.elapsed().as_millis();
        println!("Generate sequence provider: {:?}", duration);

        sequence_provider
    };
    
    print_results_of_nucleotide_only_for_local(
        fasta_file_for_reference,
        sequence_provider,
    );
}

fn print_results_of_nucleotide_only_for_local<S: SequenceProvider>(
    fasta_file_for_reference: &str,
    sequence_provider: S,
) {
    let mismatch_penalty = 4;
    let gap_open_penalty = 6;
    let gap_extend_penalty = 2;
    let minimum_aligned_length = 100;
    let penalty_per_length = 0.1;

    let kmer_size_for_lookup_table = 4;

    // For this crate
    let aligner = Aligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

    println!("Aligner: {:?}", aligner);

    let sequence_type = SequenceType::nucleotide_only();

    let mut reference = {
        let start_time = Instant::now();

        let lt_fm_index_config = LtFmIndexConfig::new()
            .change_kmer_size_for_lookup_table(kmer_size_for_lookup_table);

        let reference = Reference::new(sequence_type.clone(), lt_fm_index_config, sequence_provider).unwrap();
        
        let duration = start_time.elapsed().as_millis();
        println!("Generate reference: {:?}", duration);

        reference
    };

    // For comparison
    let standard_aligner = StandardAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length);

    println!("Standard Aligner: {:?}", standard_aligner);

    let standard_reference = {
        let start_time = Instant::now();

        let standard_reference = StandardReference::new_from_fasta(
            sequence_type,
            fasta_file_for_reference
        );

        let duration = start_time.elapsed().as_millis();
        println!("Generate standard reference: {:?}", duration);

        standard_reference
    };  

    let query_fasta_records = FastaReader::from_file_path(NUCLEOTIDE_ONLY_FA_PATH_2).unwrap();

    for (query_record_index, (label, query)) in query_fasta_records.into_iter().enumerate() {
        println!("{:?} - {:?}", query_record_index, label);

        // if query_record_index < 0 {
        //     continue
        // }

        let start_time_1 = Instant::now();
        let result_of_this_crate = aligner.local_alignment_unchecked(
            &mut reference,
            &query
        );
        let duration_1 = start_time_1.elapsed().as_millis();
    
        let start_time_2 = Instant::now();
        let result_of_standard = standard_aligner.local_alignment_raw(
            &standard_reference,
            &query
        );
        let duration_2 = start_time_2.elapsed().as_millis();

        println!("Time elapsed: {}, {}", duration_1, duration_2);

        print_alignment_results_by_index_are_same(
            &result_of_this_crate,
            &result_of_standard,
        );
    }
}

#[derive(Debug, Hash, Clone)]
struct CmpableAlignmentResult {
    penalty: usize,
    length: usize,
    position: AlignmentPosition,
}
impl CmpableAlignmentResult {
    fn alignment_results_to_hash_set_selves(alignment_results: &Vec<AlignmentResult>) -> HashSet<Self> {
        alignment_results.iter().map(|AlignmentResult {
            dissimilarity: _,
            penalty,
            length,
            position,
            operations: _,
        } | {
            Self {
                penalty: *penalty,
                length: *length,
                position: position.clone(),
            }
        }).collect()
    }
    fn alignment_results_to_sorted_selves(alignment_results: &Vec<AlignmentResult>) -> Vec<Self> {
        let mut vector_of_selves: Vec<Self> = alignment_results.iter().map(|AlignmentResult {
            dissimilarity: _,
            penalty,
            length,
            position,
            operations: _,
        } | {
            Self {
                penalty: *penalty,
                length: *length,
                position: position.clone(),
            }
        }).collect();

        vector_of_selves.sort_by(|a, b| {
            if a.penalty == b.penalty {
                if a.length == b.length {
                    if a.position.record == b.position.record {
                        a.position.query.partial_cmp(&b.position.query).unwrap()
                    } else {
                        a.position.record.partial_cmp(&b.position.record).unwrap()
                    }
                } else {
                    a.length.partial_cmp(&b.length).unwrap()
                }
            } else {
                a.penalty.partial_cmp(&b.penalty).unwrap()
            }
        });

        vector_of_selves
    }
}

impl PartialEq for CmpableAlignmentResult {
    fn eq(&self, other: &Self) -> bool {
        (
            self.penalty == other.penalty
            && self.position == other.position
        ) || (
            self.penalty == other.penalty
            && self.length == other.length
            && (
                (
                    self.position.record.0 == other.position.record.0
                    && self.position.query.0 == other.position.query.0
                ) || (
                    self.position.record.1 == other.position.record.1
                    && self.position.query.1 == other.position.query.1
                )
            )
        )
    }
}

impl Eq for CmpableAlignmentResult {}

fn assert_alignment_results_by_index_are_same(
    result_of_this_crate: &AlignmentResultsByRecordIndex,
    result_of_standard: &AlignmentResultsByRecordIndex,
) {
    let mut first_keys: Vec<usize> = result_of_this_crate.0.keys().map(|x| {*x}).collect();
    let mut second_keys: Vec<usize> = result_of_standard.0.keys().map(|x| {*x}).collect();

    first_keys.sort();
    second_keys.sort();

    // Check if keys are same
    assert_eq!(first_keys, second_keys);

    println!("Results count: {}", first_keys.len());

    for key in first_keys {
        let first_sorted_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            result_of_this_crate.0.get(&key).unwrap()
        );
        let second_sorted_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            result_of_standard.0.get(&key).unwrap()
        );

        assert_eq!(first_sorted_results, second_sorted_results);
    }
}

fn print_alignment_results_by_index_are_same(
    result_of_this_crate: &AlignmentResultsByRecordIndex,
    result_of_standard: &AlignmentResultsByRecordIndex,
) {
    let mut first_keys: Vec<usize> = result_of_this_crate.0.keys().map(|x| {*x}).collect();
    let mut second_keys: Vec<usize> = result_of_standard.0.keys().map(|x| {*x}).collect();

    first_keys.sort();
    second_keys.sort();

    // Check if keys are same
    
    if first_keys != second_keys {
        println!("Keys are different");
        println!(" - result_of_this_crate:\n{:#?}", result_of_this_crate);
        println!(" - result_of_standard:\n{:#?}", result_of_standard);
        return
    }

    println!("Results count: {}", first_keys.len());

    for key in first_keys {
        let first_results_set = CmpableAlignmentResult::alignment_results_to_hash_set_selves(
            result_of_this_crate.0.get(&key).unwrap()
        );
        let second_results_set = CmpableAlignmentResult::alignment_results_to_hash_set_selves(
            result_of_standard.0.get(&key).unwrap()
        );

        if first_results_set == second_results_set {
            println!("Same in record index: {}", key);
        } else {
            println!("Different in record index: {}", key);
            println!(" - result_of_this_crate:\n{:#?}", result_of_this_crate.0.get(&key).unwrap());
            println!(" - result_of_standard:\n{:#?}", result_of_standard.0.get(&key).unwrap());
        }
    }
}
