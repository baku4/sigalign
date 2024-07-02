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
use bio::io::fasta;
use log::info;
use sigalign::{
    Reference,
    ReferenceBuilder,
    Aligner, results::{FastaAlignment, ReadAlignment, LabeledTargetAlignment, TargetResultWithOptionalLabel,},
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
fn validate_semi_global_with_limit() {
    init_logger();

    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    let reference = ReferenceBuilder::new().add_fasta_file(&ref_file).unwrap().build().unwrap();

    let regulators = get_test_regulators();
    info!("Test {} regulators", regulators.len());

    let limits = [1000, 100, 10, 1];

    for regulator in regulators {
        info!("Regulator: {:?}", regulator);
        info!("Get result of with unlimited aligner...");
        let mut aligner = Aligner::new(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        ).unwrap();
        aligner.change_to_semi_global();

        let all_alignment_result = {
            let fasta_file = File::open(&qry_file).unwrap();
            aligner.align_fasta(&reference, fasta_file)
        };
        let all_alignment_result_set = get_set_of_fasta_alignment_result(&all_alignment_result);

        for limit in limits.iter() {
            info!("Limit: {}", limit);
            info!("Get result of with limited aligner...");
            aligner.set_limit(Some(*limit));

            let limited_alignment_result = {
                let fasta_file = File::open(&qry_file).unwrap();
                aligner.align_fasta(&reference, fasta_file)
            };
            info!("Check if limitation works...");
            limited_alignment_result.0.iter().for_each(|read_alignment_result| {
                if read_alignment_result.count_alignments() > *limit as usize {
                    panic!("Limitation does not work!");
                }
            });

            info!("Assert limited result is correct...");
            let limited_alignment_result_set = get_set_of_fasta_alignment_result(&limited_alignment_result);

            assert!(all_alignment_result_set.is_superset(&limited_alignment_result_set));
        }
    }
}

#[test]
fn validate_local_with_limit() {
    init_logger();

    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    let reference = ReferenceBuilder::new().add_fasta_file(&ref_file).unwrap().build().unwrap();

    let regulators = get_test_regulators();
    info!("Test {} regulators", regulators.len());

    let limits = [1000, 100, 10, 1];

    for regulator in regulators {
        info!("Regulator: {:?}", regulator);
        info!("Get result of with unlimited aligner...");
        let mut aligner = Aligner::new(
            regulator.0,
            regulator.1,
            regulator.2,
            regulator.3,
            regulator.4,
        ).unwrap();
        let all_alignment_result = {
            let fasta_file = File::open(&qry_file).unwrap();
            aligner.align_fasta(&reference, fasta_file)
        };
        let all_alignment_result_set = get_set_of_fasta_alignment_result(&all_alignment_result);

        for limit in limits.iter() {
            info!("Limit: {}", limit);
            info!("Get result of with limited aligner...");
            aligner.set_limit(Some(*limit));

            let limited_alignment_result = {
                let fasta_file = File::open(&qry_file).unwrap();
                aligner.align_fasta(&reference, fasta_file)
            };
            info!("Check if limitation works...");
            limited_alignment_result.0.iter().for_each(|read_alignment_result| {
                if read_alignment_result.count_alignments() > *limit as usize {
                    panic!("Limitation does not work!");
                }
            });

            info!("Assert limited result is correct...");
            let limited_alignment_result_set = get_set_of_fasta_alignment_result(&limited_alignment_result);

            assert!(all_alignment_result_set.is_superset(&limited_alignment_result_set));
        }
    }
}

fn get_set_of_fasta_alignment_result(fasta_alignment_result: &FastaAlignment) -> AHashSet<(String, u32, Alignment)> {
    let mut result_set = AHashSet::new();
    for ReadAlignment {
        read,
        is_forward: _,
        result,
    } in &fasta_alignment_result.0 {
        for TargetResultWithOptionalLabel {
            index,
            label: _,
            alignments,
        } in &result.0 {
            for alignment in alignments {
                result_set.insert((read.clone(), *index, alignment.clone()));
            }
        }
    }
    result_set
}
