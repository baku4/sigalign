use std::fs::File;

use crate::common::{
    test_data_path::{
        get_ref_for_val_path,
        get_qry_for_val_path,
        get_dir_on_tmp_dir,
    },
    init_logger,
    get_local_result_of_stable_version,
    get_semi_global_result_of_stable_version,
};
use ahash::{AHashMap, AHashSet};
use log::info;
use sigalign::{
    Reference,
    ReferenceBuilder,
    Aligner, results::{FastaAlignment, ReadAlignment, LabeledTargetAlignment},
};
use sigalign_core::results::{QueryAlignment, TargetAlignment, Alignment};

const TEST_REGULATOR_1: (u32, u32, u32, u32, f32) = (4, 6, 2, 70, 0.1);
const TEST_REGULATOR_2: (u32, u32, u32, u32, f32) = (5, 4, 3, 110, 0.2);

fn get_test_regulators() -> Vec<(u32, u32, u32, u32, f32)> {
    vec![
        TEST_REGULATOR_1,
        TEST_REGULATOR_2,
    ]
}

#[test]
fn validate_semi_global_result_with_stable_version() {
    init_logger();

    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();
    let tmp_dir = get_dir_on_tmp_dir("validate_semi_global_result_with_stable_version").unwrap();

    let reference = ReferenceBuilder::new().add_fasta_file(&ref_file).unwrap().build().unwrap();

    let regulators = get_test_regulators();
    info!("Test {} regulators", regulators.len());

    for regulator in regulators {
        info!("Regulator: {:?}", regulator);
        let local_result_answer = get_semi_global_result_of_stable_version(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
            &qry_file,
            &ref_file,
            &tmp_dir,
        );

        info!("Start alignment with current version...");
        let mut aligner = Aligner::new(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        ).unwrap();
        aligner.change_to_semi_global();

        let fasta_file = File::open(&qry_file).unwrap();
        let current_result = aligner.align_fasta(&reference, fasta_file);

        info!("Assert current result is correct...");
        assert_current_result_is_correct(current_result, local_result_answer);
    }
}

#[test]
fn validate_local_result_with_stable_version() {
    init_logger();

    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();
    let tmp_dir = get_dir_on_tmp_dir("validate_local_result_with_stable_version").unwrap();

    let reference = ReferenceBuilder::new().add_fasta_file(&ref_file).unwrap().build().unwrap();

    let regulators = get_test_regulators();
    info!("Test {} regulators", regulators.len());

    for regulator in regulators {
        info!("Regulator: {:?}", regulator);
        let local_result_answer = get_local_result_of_stable_version(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
            &qry_file,
            &ref_file,
            &tmp_dir,
        );

        info!("Start alignment with current version...");
        let mut aligner = Aligner::new(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        ).unwrap();

        let fasta_file = File::open(&qry_file).unwrap();
        let current_result = aligner.align_fasta(&reference, fasta_file);

        info!("Assert current result is correct...");
        assert_current_result_is_correct(current_result, local_result_answer);
    }
}

fn fasta_alignment_result_to_result_map(fasta_alignment_result: FastaAlignment) -> AHashMap<String, LabeledTargetAlignment> {
    let mut result_map = AHashMap::new();
    for ReadAlignment {
        read,
        is_forward: _,
        result,
    } in fasta_alignment_result.0 {
        let target_results = result.0.into_iter().map(|x| {
            TargetAlignment {
                index: x.index,
                alignments: x.alignments
            }
        }).collect();
        result_map.insert(read, QueryAlignment(target_results));
    }
    result_map
}

fn assert_current_result_is_correct(
    current_result: FastaAlignment,
    answer: AHashMap<String, LabeledTargetAlignment>,
) {
    for read_alignment_result in current_result.0 {
        let read = read_alignment_result.read;
        let current_result = read_alignment_result.result;
        let answer = answer.get(&read).unwrap();

        assert_results_are_same(&current_result, answer);
    }
}
fn assert_results_are_same(
    labeled_alignment_result: &LabeledTargetAlignment,
    answer_result: &LabeledTargetAlignment,
) {
    let result_set_1: AHashSet<(u32, Alignment)> = labeled_alignment_result.0.iter().map(|x| {
        x.alignments.iter().map(|y| {
            (x.index, y.clone())
        }).collect::<Vec<(u32, Alignment)>>()
    }).flatten().collect();
    let result_set_2: AHashSet<(u32, Alignment)> = answer_result.0.iter().map(|x| {
        x.alignments.iter().map(|y| {
            (x.index, y.clone())
        }).collect::<Vec<(u32, Alignment)>>()
    }).flatten().collect();

    assert_eq!(result_set_1, result_set_2);
}