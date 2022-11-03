// Test if result of current repository == result of stable version of SigAlign
// Answer is created from the SigAlign of version in `crate`

use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};

use super::{
    Result, error_msg,
    init_logger,
    get_ref_for_val_path,
    get_qry_for_val_path,
    get_local_tmp_dir,
};
use super::{
    ReferenceBuilder,
    Reference,
    InMemoryRcStorage,
    Aligner,
    FastaAlignmentResult,
    ReadAlignmentResult,
    AlignmentResult,
    AnchorAlignmentResult,
};

mod stable_answer;
use log::info;
use stable_answer::get_answer_or_generate;

const ANSWER_ALIGNER_OPTION: (
    usize,
    usize,
    usize,
    usize,
    f32,
) = (
    4, // Mismatch penalty
    6, // Gap-open penalty
    2, // Gap-extend penalty
    100, // Min. length
    0.1, // Max. penalty per length
);

#[test]
fn test_current_algorithms_are_collect() {
    init_logger();
    info!("Integrated test of current algorithms starts");
    
    // Prepare data
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Build reference
    let mut in_mem_rc_storage = InMemoryRcStorage::new();
    in_mem_rc_storage.add_fasta_file(&ref_file).unwrap();
    let reference = ReferenceBuilder::new().build(in_mem_rc_storage).unwrap();

    // Prepare Aligners
    let mut semi_global_aligner = Aligner::new_semi_global(
        ANSWER_ALIGNER_OPTION.0,
        ANSWER_ALIGNER_OPTION.1,
        ANSWER_ALIGNER_OPTION.2,
        ANSWER_ALIGNER_OPTION.3,
        ANSWER_ALIGNER_OPTION.4,
    ).unwrap();
    let mut local_aligner = Aligner::new_local(
        ANSWER_ALIGNER_OPTION.0,
        ANSWER_ALIGNER_OPTION.1,
        ANSWER_ALIGNER_OPTION.2,
        ANSWER_ALIGNER_OPTION.3,
        ANSWER_ALIGNER_OPTION.4,
    ).unwrap();

    info!("Reference and aligners of current are ready");

    // Perform alignment
    let [semi_global_result_answer, local_result_answer] = get_answer_or_generate().unwrap();
    info!("Answers are loaded");
    let semi_global_result_of_current = semi_global_aligner.fasta_file_alignment(&reference, &qry_file).unwrap();
    info!("Alignment of semi-global mode is done");
    compare_fasta_alignment_result(semi_global_result_of_current, semi_global_result_answer);
    info!("Comparison of semi-global mode is done");
    let local_result_of_current = local_aligner.fasta_file_alignment(&reference, &qry_file).unwrap();
    info!("Alignment of local mode is done");
    compare_fasta_alignment_result(local_result_of_current, local_result_answer);
    info!("Comparison of local mode is done");
}

fn compare_fasta_alignment_result(mut a: FastaAlignmentResult, mut b: FastaAlignmentResult) {
    // Sort by read
    let sort_by_read = |a: &ReadAlignmentResult, b: &ReadAlignmentResult| a.read.cmp(&b.read);
    a.0.sort_by(sort_by_read);
    b.0.sort_by(sort_by_read);

    assert_eq!(a.0.len(), b.0.len());
    
    for (read_res_a, read_res_b) in a.0.into_iter().zip(b.0.into_iter()) {
        // Read name
        assert_eq!(read_res_a.read, read_res_b.read);
        let res_a = read_res_a.result;
        let res_b = read_res_b.result;

        // Record Result
        let sorted_comp_res_a = change_alignment_result_to_sorted_results(res_a);
        let sorted_comp_res_b = change_alignment_result_to_sorted_results(res_b);
        assert_eq!(sorted_comp_res_a.len(), sorted_comp_res_b.len());

        for (record_res_a, record_res_b) in sorted_comp_res_a.into_iter().zip(sorted_comp_res_b.into_iter()) {
            assert_eq!(record_res_a.index, record_res_b.index);
            assert_eq!(record_res_a.alignments_set, record_res_b.alignments_set);
        }
    }
}

use ahash::AHashSet;
struct CompRecordResult {
    index: usize,
    alignments_set: AHashSet<AnchorAlignmentResult>,
}
fn change_alignment_result_to_sorted_results(result: AlignmentResult) -> Vec<CompRecordResult> {
    let record_alignment_results = result.0;
    let mut results: Vec<CompRecordResult> = record_alignment_results.into_iter().map(|record_alignment_result| {
        let index = record_alignment_result.index;
        let alignments_set: AHashSet<AnchorAlignmentResult> = record_alignment_result.alignments.into_iter().collect();
        CompRecordResult {
            index,
            alignments_set,
        }
    }).collect();
    results.sort_by(|a, b| {a.index.cmp(&b.index)});

    results
}
