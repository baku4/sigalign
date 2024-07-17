use log::{error, info};
use ahash::AHashSet as HashSet;
use crate::common::{
    init_logger,
    test_data::DataForValidation,
    configuration::TestSetting,
    random_regulator::{gen_random_regulator_not_errored_in_v03},
    result_converter_of_v03::stable_result_to_current_result,
    dynamic_programming_matrix::{
        dp_local_with_one_mat_to_target,
        dp_semi_global_to_target,
    },
};
use sigalign_utils::sequence_reader::{
    SeqRecord,
    SeqRefRecord,
    IdRecord,
    IdRefRecord,
    fasta::{FastaReader, FastaRecord},
    fastq::{FastqReader, FastqRecord},
};
use sigalign::{
    algorithms::{Algorithm, Local, SemiGlobal}, results::{Alignment, QueryAlignment, TargetAlignment}, Aligner as CurrentAligner, Reference as CurrentReference, ReferenceBuilder
};
use sigalign_stable::{
    results::AlignmentResult as StableQueryAlignment,
    wrapper::{
        DefaultAligner as StableAligner,
        DefaultReference as StableReference,
    },
};

mod check_results;
use check_results::{
    compare_the_results_are_the_same_and_return_errored_target_index,
    sort_alignments_by_optimality,
};
mod dpm_results_cache;
use dpm_results_cache::{DpmAlignerWithCache, DpmMode};

const MAX_MISMATCH_PER_100_BASES: u32 = 3;
const REGULATOR_START_SEED: u64 = 0;
const REGULATOR_SEED_COUNT: u64 = 2;

// Compare current results with stable version.
// If the two results are different, using DPM to check the results.
#[test]
fn test_local_is_equal_to_stable_or_dpm() {
    init_logger();

    // Prepare paths
    let test_data = DataForValidation::Default;
    let (ref_file, qry_file) = test_data.get_data_paths();

    // Prepare reference
    info!("Start to prepare reference");
    let current_reference = ReferenceBuilder::new()
        .add_fasta_file(&ref_file).unwrap()
        .build().unwrap();
    let stable_reference = StableReference::from_fasta_file(
        &ref_file
    ).unwrap();

    // Start to compare
    for seed in REGULATOR_START_SEED..REGULATOR_START_SEED + REGULATOR_SEED_COUNT {
        let regulators = gen_random_regulator_not_errored_in_v03(
            MAX_MISMATCH_PER_100_BASES,
            seed,
        );
        info!("Start to compare with regulators: {:?} (seed: {})", regulators, seed);

        let algorithm = Local::new(
            regulators.0,
            regulators.1,
            regulators.2,
            regulators.3,
            regulators.4,
        ).unwrap();
        let mut current_aligner = CurrentAligner::from(algorithm);
        let mut stable_aligner = StableAligner::new_local(
            regulators.0,
            regulators.1,
            regulators.2,
            regulators.3,
            regulators.4,
        ).unwrap();

        let mut fasta_reader = FastaReader::new(
            std::fs::File::open(&qry_file).unwrap()
        );
        let mut query_buffer = Vec::new();
        let mut query_index = 0;
        while let Some(mut record) = fasta_reader.next() {
            query_buffer.clear();
            if query_index % 100 == 0 {
                info!("Processed {} queries", query_index);
            }
            record.extend_seq_buf(&mut query_buffer);

            // Get current result
            let current_result = current_aligner.align(&query_buffer, &current_reference);

            // Get stable result
            let stable_result = stable_aligner.align_query(&stable_reference, &query_buffer);
            let stable_result = stable_result_to_current_result(stable_result);

            // Compare
            let errored_target_index = compare_the_results_are_the_same_and_return_errored_target_index(
                &current_result,
                &stable_result,
            );
            if !errored_target_index.is_empty() {
                info!("[Query index: {}] Errored target index: {:?}", query_index, errored_target_index);
            }

            // Check the errored target index using DPM
            for target_index in errored_target_index.iter() {
                let target = current_reference.get_sequence(*target_index).unwrap();

                let current_dedup_alignments = {
                    let mut target_alignment = None;
                    for v in current_result.0.iter() {
                        if v.index == *target_index {
                            target_alignment = Some(v.clone());
                            break;
                        }
                    }
                    target_alignment.unwrap().alignments
                };
                let dpm_alignments = {
                    let dpm_aligner = DpmAlignerWithCache::new(
                        DpmMode::LocalWithAllSubs,
                        test_data.get_tag().to_string(),
                        query_index,
                        *target_index,
                        regulators.0,
                        regulators.1,
                        regulators.2,
                        regulators.3,
                        regulators.4,
                    );
                    dpm_aligner.perform_alignment_if_needed(&query_buffer, &target).alignments
                };
                
                let mut to_compare_current = {
                    TargetAlignment {
                        index: *target_index,
                        alignments: current_dedup_alignments.clone(),
                    }.deduplicated().alignments
                };
                let mut to_compare_dpm = dpm_alignments.clone();
                sort_alignments_by_optimality(&mut to_compare_current);
                remove_operations(&mut to_compare_current);
                sort_alignments_by_optimality(&mut to_compare_dpm);
                remove_operations(&mut to_compare_dpm);

                if to_compare_current != to_compare_dpm {
                    let set_current: HashSet<Alignment> = current_dedup_alignments.iter().cloned().collect();
                    let set_dpm: HashSet<Alignment> = dpm_alignments.iter().cloned().collect();
                    let only_in_current = set_current.difference(&set_dpm).collect::<Vec<_>>();
                    let only_in_dpm = set_dpm.difference(&set_current).collect::<Vec<_>>();
                    if only_in_dpm.len() == 0 {
                        info!(
                            "[Query index: {}] Target index: {} is not equal to DPM, but allowed since current is superset of DPM",
                            query_index, target_index
                        );
                    } else {
                        error!(
                            "[Query index: {}] Target index: {} is not equal to DPM, and current is not superset of DPM",
                            query_index, target_index,
                        );
                        error!(" - Query: {}", String::from_utf8_lossy(&query_buffer));
                        error!(" - Target: {}", String::from_utf8_lossy(&target));
                        error!(" - Current results: {:?}", current_dedup_alignments);
                        error!(" - DPM results: {:?}", dpm_alignments);
                        
                        error!(" - Only in current: {:?}", only_in_current);
                        error!(" - Only in DPM: {:?}", only_in_dpm);
                        
                        panic!();
                    }
                } else {
                    info!("[Query index: {}] Target index: {} is equal to DPM", query_index, target_index);
                }
            }

            query_index += 1;
        }
    }
}

fn remove_operations(alignments: &mut Vec<Alignment>) {
    alignments.iter_mut().for_each(|x| {
        x.operations.clear();
    });
}
