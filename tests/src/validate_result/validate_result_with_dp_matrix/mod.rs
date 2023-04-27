use std::ops::Sub;

use crate::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};
use ahash::AHashSet;
use sigalign::{
    wrapper::{
        DefaultAligner, DefaultReference,
    }, utils::FastaReader, results::{AlignmentResult, AnchorAlignmentResult, AlignmentPosition},
};
use log::{info, error};

mod generate_answer_with_dp_matrix;
use generate_answer_with_dp_matrix::{
    get_cached_semi_global_result_with_dp_matrix,
};
mod generate_all_answer_with_threads;

const ALIGNER_OPTION: (
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
fn validate_semi_global_mode_with_dp_matrix() {
    let qry_count = 10000; // TODO: Use Total Qry

    init_logger();
    info!("Start to validate semi-global result with DP matrix");
    
    // Prepare data
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Build reference
    let reference = DefaultReference::from_fasta_file(&ref_file).unwrap();

    // Prepare Aligners
    let mut semi_global_aligner = DefaultAligner::new_semi_global(
        ALIGNER_OPTION.0,
        ALIGNER_OPTION.1,
        ALIGNER_OPTION.2,
        ALIGNER_OPTION.3,
        ALIGNER_OPTION.4,
    ).unwrap();
    info!("Reference and aligners of current are ready");

    // Perform alignment
    let qry_reader = FastaReader::from_path(qry_file).unwrap();
    for (qry_index, (label, query)) in qry_reader.into_iter().enumerate() {
        info!(" - query label: {}", label);
        if qry_index == qry_count { break };

        let dpm_result = get_cached_semi_global_result_with_dp_matrix(
            &query,
            &label,
            &ref_file,
            ALIGNER_OPTION.0,
            ALIGNER_OPTION.1,
            ALIGNER_OPTION.2,
            ALIGNER_OPTION.3,
            ALIGNER_OPTION.4,
        );

        let sigalign_result = semi_global_aligner.align_query(&reference, &query).unwrap();

        assert_sigalign_result_includes_the_dpm_result(&sigalign_result, &dpm_result);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct EquivalentResult {
    target_index: u32,
    penalty: u32,
    length: u32,
    position: AlignmentPosition,
}

fn assert_sigalign_result_includes_the_dpm_result(
    sigalign_result: &AlignmentResult,
    dpm_result: &AlignmentResult,
) {
    let sigalign_result_set: AHashSet<EquivalentResult> = alignment_result_to_hashset(sigalign_result);

    let dpm_result_set: AHashSet<EquivalentResult> = alignment_result_to_hashset(dpm_result);

    // assert!(sigalign_result_set.is_superset(&dpm_result_set));
    if !sigalign_result_set.is_superset(&dpm_result_set) {
        println!("#dpm_result:\n{:#?}", &dpm_result);
        println!("#sigalign_result:\n{:#?}", &sigalign_result);
        println!("#only in sigalign:\n{:#?}", &sigalign_result_set.sub(&dpm_result_set));
        println!("#only in dpm:\n{:#?}", &dpm_result_set.sub(&sigalign_result_set));
        panic!("SigAlign result does not contain the DPM result");
    }
}

fn alignment_result_to_hashset(result: &AlignmentResult) -> AHashSet<EquivalentResult> {
    result.0.iter().map(|x| {
        x.alignments.iter().map(|y| {
            EquivalentResult {
                target_index: x.index,
                penalty: y.penalty,
                length: y.length,
                position: y.position.clone(),
            }
        })
    }).flatten().collect()
}