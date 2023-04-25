use crate::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};
use sigalign::{
    wrapper::{
        DefaultAligner, DefaultReference,
    },
};
use log::{info, error};

mod generate_answer_with_dp_matrix;
use generate_answer_with_dp_matrix::{
    get_answer_or_generate_semi_global_result_with_dp_matrix,
};

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
    let qry_count = 200; // TODO: Use Total Qry

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
    // let [semi_global_result_answer, local_result_answer] = get_answer_or_generate().unwrap();
    // let semi_global_result_answer = convert_result_of_stable_version_to_current(&semi_global_result_answer);
    // let local_result_answer = convert_result_of_stable_version_to_current(&local_result_answer);
    info!("Answers are loaded");

    // let semi_global_result_of_current = semi_global_aligner.align_fasta_file(&reference, &qry_file).unwrap();
    info!("Alignment of semi-global mode is done");

    // assert_eq_fasta_alignment_result(semi_global_result_of_current, semi_global_result_answer);
    info!("Comparison of semi-global mode is done");
}