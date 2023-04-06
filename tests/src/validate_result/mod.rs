use std::collections::HashSet;
// Test if result of current repository == result of stable version of SigAlign
// Answer is created from the SigAlign of version in `crate`
use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};

use crate::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};

use sigalign::results::AlignmentOperation;
use sigalign::utils::FastaReader;
use sigalign::{
    wrapper::{
        DefaultReference, DefaultAligner
    },
    results::{
        fasta::{FastaAlignmentResult, ReadAlignmentResult},
        AlignmentResult,
        TargetAlignmentResult,    
        AnchorAlignmentResult,
    },
};

use log::{info, error};
mod stable_answer;
mod result_converter;
use stable_answer::get_answer_or_generate;
use result_converter::convert_result_of_stable_version_to_current;

// For bench
pub fn get_sample_result_of_val_test() -> FastaAlignmentResult {
    let [sample_result_of_stable, _] = get_answer_or_generate().unwrap();
    let sample_result = convert_result_of_stable_version_to_current(&sample_result_of_stable);
    sample_result
}

const ANSWER_ALIGNER_OPTION: (
    u32,
    u32,
    u32,
    u32,
    f32,
) = (
    4,   // Mismatch penalty
    6,   // Gap-open penalty
    2,   // Gap-extend penalty
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
    let reference = DefaultReference::from_fasta_file(&ref_file).unwrap();

    // Prepare Aligners
    let mut semi_global_aligner = DefaultAligner::new_semi_global(
        ANSWER_ALIGNER_OPTION.0,
        ANSWER_ALIGNER_OPTION.1,
        ANSWER_ALIGNER_OPTION.2,
        ANSWER_ALIGNER_OPTION.3,
        ANSWER_ALIGNER_OPTION.4,
    ).unwrap();
    let mut local_aligner = DefaultAligner::new_local(
        ANSWER_ALIGNER_OPTION.0,
        ANSWER_ALIGNER_OPTION.1,
        ANSWER_ALIGNER_OPTION.2,
        ANSWER_ALIGNER_OPTION.3,
        ANSWER_ALIGNER_OPTION.4,
    ).unwrap();

    info!("Reference and aligners of current are ready");

    // Perform alignment
    let [semi_global_result_answer, local_result_answer] = get_answer_or_generate().unwrap();
    let semi_global_result_answer = convert_result_of_stable_version_to_current(&semi_global_result_answer);
    let local_result_answer = convert_result_of_stable_version_to_current(&local_result_answer);
    info!("Answers are loaded");

    let semi_global_result_of_current = semi_global_aligner.align_fasta_file(&reference, &qry_file).unwrap();
    info!("Alignment of semi-global mode is done");

    assert_eq_fasta_alignment_result(semi_global_result_of_current, semi_global_result_answer);
    info!("Comparison of semi-global mode is done");

    let local_result_of_current = local_aligner.align_fasta_file(&reference, &qry_file).unwrap();
    info!("Alignment of local mode is done");
    
    assert_eq_fasta_alignment_result(local_result_of_current, local_result_answer);
    info!("Comparison of local mode is done");
}
#[test]
fn print_the_first_alignment_result_for_local() {
    init_logger();
    info!("Integrated test of current algorithms starts");
    
    // Prepare data
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Build reference
    let reference = DefaultReference::from_fasta_file(&ref_file).unwrap();

    // Prepare Aligners
    let mut local_aligner = DefaultAligner::new_local(
        ANSWER_ALIGNER_OPTION.0,
        ANSWER_ALIGNER_OPTION.1,
        ANSWER_ALIGNER_OPTION.2,
        ANSWER_ALIGNER_OPTION.3,
        ANSWER_ALIGNER_OPTION.4,
    ).unwrap();

    info!("Reference and aligners of current are ready");

    // Perform alignment
    let [_, local_result_answer] = get_answer_or_generate().unwrap();
    let local_result_answer = convert_result_of_stable_version_to_current(&local_result_answer);
    info!("Answers are loaded");

    //
    // To view result
    //
    let result_index_list = 0..=5000;
    for result_index in result_index_list {
        let one_read = local_result_answer.0[result_index].read.clone();
        let one_local_result_answer = local_result_answer.0[result_index].result.clone();
        
        let one_query = {
            let mut fasta_reader = FastaReader::from_path(&qry_file).unwrap();
            loop {
                let query = fasta_reader.next().unwrap();
                if query.0 == one_read {
                    break query
                }
            }
        };

        // println!("# result_index: {:?}", result_index);
        // println!("# query:\n{:?}", String::from_utf8(one_query.1.clone()).unwrap());

        let one_local_result_of_current = local_aligner.align_query(
            &reference,
            &one_query.1,
        ).unwrap();
        
        info!("Alignment of local mode is done");

        // if !right_result_includes_left(&one_local_result_answer, &one_local_result_of_current) {
        //     let res_count = |res: &AlignmentResult| {
        //         res.0.iter().map(|x| x.alignments.len()).sum::<usize>()
        //     };
        //     println!(
        //         "result count: answer-{}, current-{}",
        //         res_count(&one_local_result_answer),
        //         res_count(&one_local_result_of_current),
        //     );
        // }

        // Check if last operation is INS
        one_local_result_of_current.0.iter().for_each(|target_result| {
            target_result.alignments.iter().for_each(|x| {
                let ops = &x.operations;
                let op = ops[0].operation.clone();
                if (op == AlignmentOperation::Insertion) {
                    println!("First op is INS");
                    println!("target_result:\n{:?}", target_result);
                    panic!("");
                }
                let op = ops.last().unwrap().operation.clone();
                if (op == AlignmentOperation::Insertion) {
                    println!("Last op is INS");
                    println!("target_result:\n{:?}", target_result);
                    panic!("");
                }
            })
        });
    }
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
    let sorted_a = sort_target_alignment_results(&a.0);
    let sorted_b = sort_target_alignment_results(&b.0);

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
fn right_result_includes_left(
    a: &AlignmentResult,
    b: &AlignmentResult,
) -> bool {
    let to_set = |alignment_result: &AlignmentResult| {
        let set: HashSet<(u32, AnchorAlignmentResult)> = alignment_result.0.iter().map(|target_alignment_result| {
            let target_index = &target_alignment_result.index;
            target_alignment_result.alignments.iter().map(|res| {
                (*target_index, res.clone())
            })
        }).flatten().collect();
        set
    };
    let set_a = to_set(a);
    let set_b = to_set(b);
    set_a.is_subset(&set_b)
}
fn sort_target_alignment_results(vec: &Vec<TargetAlignmentResult>) -> Vec<TargetAlignmentResult> {
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
                    let order5 = a.position.target.0.cmp(&b.position.target.0);
                    if let Ordering::Equal = order5 {
                        a.position.target.1.cmp(&b.position.target.1)
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