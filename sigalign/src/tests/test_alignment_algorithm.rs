use super::*;

use std::collections::{HashSet, HashMap};
use std::collections::hash_map::RandomState;
use std::collections::hash_set::Intersection;
use std::time::{Duration, Instant};

fn test_local_results_include_semi_global_results_with_in_memory_provider() {
    println!("Use In-Memory Provider");

    let fasta_file_for_reference = NUCLEOTIDE_ONLY_FA_PATH_1;

    let sequence_provider = {
        let start_time = Instant::now();
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_fasta_file(fasta_file_for_reference).unwrap();
        let duration = start_time.elapsed().as_micros();
        println!("Generate sequence provider: {:?}", duration);

        sequence_provider
    };

    let mismatch_penalty = 4;
    let gap_open_penalty = 6;
    let gap_extend_penalty = 2;
    let minimum_aligned_length = 100;
    let penalty_per_length = 0.1;

    let kmer_size_for_lookup_table = 4;

    // For this crate
    let mut local_aligner = Aligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();
    let mut semi_global_aligner = Aligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

    println!("Local Aligner: {:?}", local_aligner);
    println!("Semi Global Aligner: {:?}", semi_global_aligner);

    let mut reference = {
        let start_time = Instant::now();

        let reference = ReferenceBuilder::new()
            .search_for_nucleotide_only()
            .change_count_array_kmer(kmer_size_for_lookup_table).unwrap()
            .build(sequence_provider).unwrap();
        
        let duration = start_time.elapsed().as_micros();
        println!("Generate reference: {:?}", duration);

        reference
    };

    let query_fasta_records = FastaReader::from_file_path(NUCLEOTIDE_ONLY_FA_PATH_2).unwrap();

    for (query_record_index, (label, query)) in query_fasta_records.into_iter().enumerate() {
        println!("{:?} - {:?}", query_record_index, label);

        let start_time_1 = Instant::now();
        let result_of_semi_global = semi_global_aligner.query_alignment_unchecked(&reference, &query);
        let duration_1 = start_time_1.elapsed().as_micros();
    
        let start_time_2 = Instant::now();
        let result_of_local = local_aligner.query_alignment_unchecked(&reference, &query);

        let duration_2 = start_time_2.elapsed().as_micros();

        println!("Time elapsed: {}, {}", duration_1, duration_2);

        print_if_left_result_dont_include_right_result(
            &result_of_local,
            &result_of_semi_global,
        );
    }
}

fn test_results_of_nucleotide_only_for_semi_global_with_in_memory_provider() {
    println!("Use In-Memory Provider");

    let fasta_file_for_reference = NUCLEOTIDE_ONLY_FA_PATH_1;

    let sequence_provider = {
        let start_time = Instant::now();
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_fasta_file(fasta_file_for_reference).unwrap();

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
    let mut semi_global_aligner = Aligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

    println!("Aligner: {:?}", semi_global_aligner);

    let mut reference = {
        let start_time = Instant::now();

        let reference = ReferenceBuilder::new()
            .search_for_nucleotide_only()
            .change_count_array_kmer(kmer_size_for_lookup_table).unwrap()
            .build(sequence_provider).unwrap();
        
        let duration = start_time.elapsed().as_micros();
        println!("Generate reference: {:?}", duration);

        reference
    };

    // For comparison
    let standard_aligner = StandardAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length);

    println!("Standard Aligner: {:?}", standard_aligner);

    let standard_reference = {
        let start_time = Instant::now();

        let standard_reference = StandardReference::new_from_fasta(
            SequenceType::new_nucleotide_only(),
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
        let result_of_this_crate = semi_global_aligner.query_alignment_unchecked(
            &reference,
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

fn test_results_of_nucleotide_only_for_local_with_in_memory_provider() {
    println!("Use In-Memory Provider");

    let fasta_file_for_reference = NUCLEOTIDE_ONLY_FA_PATH_1;

    let sequence_provider = {
        let start_time = Instant::now();
        let mut sequence_provider = InMemoryProvider::new();
        sequence_provider.add_fasta_file(fasta_file_for_reference).unwrap();

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
    let mut local_aligner = Aligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length).unwrap();

    println!("Aligner: {:?}", local_aligner);

    let mut reference = {
        let start_time = Instant::now();

        let reference = ReferenceBuilder::new()
            .search_for_nucleotide_only()
            .change_count_array_kmer(kmer_size_for_lookup_table).unwrap()
            .build(sequence_provider).unwrap();
        
        let duration = start_time.elapsed().as_micros();
        println!("Generate reference: {:?}", duration);

        reference
    };

    // For comparison
    let standard_aligner = StandardAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, penalty_per_length);

    println!("Standard Aligner: {:?}", standard_aligner);

    let standard_reference = {
        let start_time = Instant::now();

        let standard_reference = StandardReference::new_from_fasta(
            SequenceType::new_nucleotide_only(),
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
        let result_of_this_crate = local_aligner.query_alignment_unchecked(
            &reference,
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
    fn alignment_results_to_hash_set_selves(alignment_results: &Vec<AnchorAlignmentResult>) -> HashSet<Self> {
        alignment_results.iter().map(|AnchorAlignmentResult {
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
    fn alignment_results_to_sorted_selves(alignment_results: &Vec<AnchorAlignmentResult>) -> Vec<Self> {
        let mut vector_of_selves: Vec<Self> = alignment_results.iter().map(|AnchorAlignmentResult {
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
    result_of_this_crate: &AlignmentResult,
    result_of_standard: &AlignmentResult,
) {
    let mut first_keys: Vec<usize> = result_of_this_crate.0.iter().map(|x| x.index).collect();
    let mut second_keys: Vec<usize> = result_of_standard.0.iter().map(|x| x.index).collect();

    first_keys.sort();
    second_keys.sort();

    // Check if keys are same
    assert_eq!(first_keys, second_keys);

    println!("Results count: {}", first_keys.len());

    let result_of_this_crate_map: HashMap<usize, Vec<AnchorAlignmentResult>> = result_of_this_crate.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();
    let result_of_standard_map: HashMap<usize, Vec<AnchorAlignmentResult>> = result_of_standard.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();

    for key in first_keys {
        let first_sorted_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            result_of_this_crate_map.get(&key).unwrap()
        );
        let second_sorted_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            result_of_standard_map.get(&key).unwrap()
        );

        assert_eq!(first_sorted_results, second_sorted_results);
    }
}

fn print_alignment_results_by_index_are_same(
    result_of_this_crate: &AlignmentResult,
    result_of_standard: &AlignmentResult,
) {
    let mut first_keys: Vec<usize> = result_of_this_crate.0.iter().map(|x| x.index).collect();
    let mut second_keys: Vec<usize> = result_of_standard.0.iter().map(|x| x.index).collect();

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

    let result_of_this_crate_map: HashMap<usize, Vec<AnchorAlignmentResult>> = result_of_this_crate.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();
    let result_of_standard_map: HashMap<usize, Vec<AnchorAlignmentResult>> = result_of_standard.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();

    for key in first_keys {
        let first_results_set = CmpableAlignmentResult::alignment_results_to_hash_set_selves(
            result_of_this_crate_map.get(&key).unwrap()
        );
        let second_results_set = CmpableAlignmentResult::alignment_results_to_hash_set_selves(
            result_of_standard_map.get(&key).unwrap()
        );

        if first_results_set == second_results_set {
            println!("Same in record index: {}", key);
        } else {
            println!("Different in record index: {}", key);
            println!(" - result_of_this_crate:\n{:#?}", result_of_this_crate_map.get(&key).unwrap());
            println!(" - result_of_standard:\n{:#?}", result_of_standard_map.get(&key).unwrap());
        }
    }
}

fn print_if_left_result_dont_include_right_result(
    left_result: &AlignmentResult,
    right_result: &AlignmentResult,
) {
    let left_keys: HashSet<usize> = left_result.0.iter().map(|x| x.index).collect();
    let right_keys: HashSet<usize> = right_result.0.iter().map(|x| x.index).collect();

    let intersection_keys: HashSet<usize> = {
        let intersection = left_keys.intersection(&right_keys);
        intersection.map(|x| {*x}).collect()
    };

    assert_eq!(intersection_keys, right_keys);

    let left_result_map: HashMap<usize, Vec<AnchorAlignmentResult>> = left_result.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();
    let right_result_map: HashMap<usize, Vec<AnchorAlignmentResult>> = right_result.0.iter()
        .map(|record_alignment_result| {
            (record_alignment_result.index, record_alignment_result.result.clone())
        }).collect();

    for key in intersection_keys {
        let left_cmpable_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            left_result_map.get(&key).unwrap()
        );
        let right_cmpable_results = CmpableAlignmentResult::alignment_results_to_sorted_selves(
            right_result_map.get(&key).unwrap()
        );

        let left_cmpable_results_set: HashSet<CmpableAlignmentResult> = left_cmpable_results.into_iter().collect();
        let right_cmpable_results_set: HashSet<CmpableAlignmentResult> = right_cmpable_results.into_iter().collect();

        let intersection_cmpable_results_set: HashSet<CmpableAlignmentResult> = {
            let intersection = left_cmpable_results_set.intersection(&right_cmpable_results_set);
            intersection.map(|x| { x.clone() }).collect()
        };

        if intersection_cmpable_results_set != right_cmpable_results_set {
            println!("{:#?}", left_result);
            println!("{:#?}", right_result);
        }
    }
}