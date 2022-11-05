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
    RecordAlignmentResult,
    AlignmentResult,
    AnchorAlignmentResult,
};

mod stable_answer;
use log::{info, error};
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
    assert_eq_fasta_alignment_result(semi_global_result_of_current, semi_global_result_answer);
    info!("Comparison of semi-global mode is done");
    let local_result_of_current = local_aligner.fasta_file_alignment(&reference, &qry_file).unwrap();
    info!("Alignment of local mode is done");
    assert_eq_fasta_alignment_result(local_result_of_current, local_result_answer);
    info!("Comparison of local mode is done");
}

fn assert_eq_fasta_alignment_result(mut a: FastaAlignmentResult, mut b: FastaAlignmentResult) {
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

        let is_equal = is_equal_alignment_result(&res_a, &res_b);
        if !is_equal {
            error!("Unequal result in query {}.", &read_res_a.read);    
        }
        assert!(is_equal);
    }
}

use std::cmp::Ordering;
fn is_equal_alignment_result(
    a: &AlignmentResult,
    b: &AlignmentResult,
) -> bool {
    let sorted_a = sort_record_alignment_results(&a.0);
    let sorted_b = sort_record_alignment_results(&b.0);

    if sorted_a.len() != sorted_b.len() {
        error!("Unequal record alignment count. left: {}, right: {}", sorted_a.len(), sorted_b.len());
        false
    } else {
        sorted_a.into_iter().zip(sorted_b.into_iter()).all(|(a,b)| {
            if a.index == b.index {
                if is_equal_anchor_alignment_results(&a.alignments, &b.alignments) {
                    true
                } else {
                    error!("Unequal in record index {}", a.index);
                    false
                }
            } else {
                error!("Unequal record index. left: {}, right: {}", a.index, b.index);
                false
            }
        })
    }
}
fn sort_record_alignment_results(vec: &Vec<RecordAlignmentResult>) -> Vec<RecordAlignmentResult> {
    let mut sorted = vec.clone();
    sorted.sort_by_key(|v| v.index);
    sorted
}
fn is_equal_anchor_alignment_results(
    a: &Vec<AnchorAlignmentResult>,
    b: &Vec<AnchorAlignmentResult>,
) -> bool {
    let sorted_a = sort_anchor_alignment_results(a);
    let sorted_b = sort_anchor_alignment_results(b);

    if sorted_a.len() != sorted_b.len() {
        error!("Unequal anchor alignment count. left: {}, right: {}", sorted_a.len(), sorted_b.len());
        false
    } else {
        let mut is_equal = true;
        for (a, b) in sorted_a.iter().zip(sorted_b.iter()) {
            let is_equal_anchor_alignment_result = {
                a.penalty == b.penalty
                && a.length == b.length
                && a.position == b.position
            };
            if !is_equal_anchor_alignment_result {
                error!("Unequal anchor alignment result. left: {:?}, right: {:?}", sorted_a, sorted_b);
                is_equal = false;
                break;
            }
        }
        is_equal
    }
}
fn sort_anchor_alignment_results(vec: &Vec<AnchorAlignmentResult>) -> Vec<AnchorAlignmentResult> {
    let mut sorted = vec.clone();
    sorted.sort_by(|a, b| cmp_anchor_alignment_result(a, b));
    sorted
}
fn cmp_anchor_alignment_result(a: &AnchorAlignmentResult, b: &AnchorAlignmentResult) -> Ordering {
    let order1 = a.penalty.cmp(&b.penalty);
    if let Ordering::Equal = order1 {
        let order2 = a.length.cmp(&b.length);
        if let Ordering::Equal = order2 {
            let order3 = a.position.query.0.cmp(&b.position.query.0);
            if let Ordering::Equal = order3 {
                let order4 = a.position.query.1.cmp(&b.position.query.1);
                if let Ordering::Equal = order4 {
                    let order5 = a.position.record.0.cmp(&b.position.record.0);
                    if let Ordering::Equal = order5 {
                        a.position.record.1.cmp(&b.position.record.1)
                    } else {
                        order5
                    }
                } else {
                    order4
                }
            } else {
                order3
            }
        } else {
            order2
        }
    } else {
        order1
    }
}