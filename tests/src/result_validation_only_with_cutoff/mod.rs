use log::{error, info};
use crate::common::{
    init_logger,
    test_data_path::{get_qry_for_val_path, get_ref_for_val_path},
    random_regulator::gen_random_regulator,
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

// Test options:
//   - Aligner's options to test
#[cfg(feature = "ci")]
const ALIGNER_OPTION_COUNT: u32 = 2;
#[cfg(not(feature = "ci"))]
const ALIGNER_OPTION_COUNT: u32 = 10;
//  - Complexity
#[cfg(feature = "ci")]
const ASSUMED_MAX_MISMATCH_PER_100: u32 = 2;
#[cfg(not(feature = "ci"))]
const ASSUMED_MAX_MISMATCH_PER_100: u32 = 4;
//  - Query interval
#[cfg(feature = "ci")]
const QUERY_SAMPLING_INTERVAL: u32 = 100;
#[cfg(not(feature = "ci"))]
const QUERY_SAMPLING_INTERVAL: u32 = 1;

#[test]
fn test_local_mode_gives_valid_results_for_cutoffs() {
    let current_aligner_generator = |px, po, pe, minl, maxp| {
        CurrentAligner::new(
            Local::new(px, po, pe, minl, maxp).unwrap()
        )
    };
    let regulators: Vec<(u32, u32, u32, u32, f32)> = (0..ALIGNER_OPTION_COUNT)
        .map(|_| gen_random_regulator(ASSUMED_MAX_MISMATCH_PER_100)).collect::<Vec<_>>();

    test_of_current_algorithm(
        &current_aligner_generator,
        regulators,
    );
}

#[test]
fn test_semi_global_mode_gives_valid_results_for_cutoffs() {
    let current_aligner_generator = |px, po, pe, minl, maxp| {
        CurrentAligner::new(
            SemiGlobal::new(px, po, pe, minl, maxp).unwrap()
        )
    };
    let regulators: Vec<(u32, u32, u32, u32, f32)> = (0..ALIGNER_OPTION_COUNT)
        .map(|_| gen_random_regulator(ASSUMED_MAX_MISMATCH_PER_100)).collect::<Vec<_>>();

    test_of_current_algorithm(
        &current_aligner_generator,
        regulators,
    );
}

fn test_of_current_algorithm<F, A>(
    current_aligner_generator: &F,
    regulators: Vec<(u32, u32, u32, u32, f32)>,
) where
    A: Algorithm,
    F: Fn(u32, u32, u32, u32, f32) -> CurrentAligner<A>,
{
    init_logger();

    // Prepare paths
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Prepare reference
    info!("Start to prepare reference");
    let current_reference = ReferenceBuilder::new()
        .add_fasta_file(&ref_file).unwrap()
        .build().unwrap();

    // Start to compare
    for aligner_option in regulators {
        // Time to compare
        let mut current_times = Vec::new();
        info!("Start to validate with aligner option: {:?}", aligner_option);
        let mut current_aligner = current_aligner_generator(
            aligner_option.0,
            aligner_option.1,
            aligner_option.2,
            aligner_option.3,
            aligner_option.4,
        );

        let mut fasta_reader = FastaReader::new(
            std::fs::File::open(&qry_file).unwrap()
        );
        let mut query_buffer = Vec::new();
        let mut query_index = 0;
        let mut query_step = 0;
        while let Some(mut record) = fasta_reader.next() {
            query_step += 1;
            if query_step == QUERY_SAMPLING_INTERVAL {
                query_step = 0;
            } else {
                continue;
            }
            
            query_buffer.clear();
            if query_index % 1000 == 0 {
                info!("Processed {} queries", query_index);
            }
            record.extend_seq_buf(&mut query_buffer);

            // Current
            let current_result = {
                let start = std::time::Instant::now();
                let result = current_aligner.align(&query_buffer, &current_reference);
                let duration = start.elapsed();
                current_times.push(duration);
                result
            };

            // Check the results
            for target_alignment in current_result.0.iter() {
                for alignment in &target_alignment.alignments {
                    let penalty = alignment.penalty;
                    let length = alignment.length;
                    let valid = (
                        (penalty as f32 / length as f32) <= aligner_option.4
                    ) && (
                        length >= aligner_option.3
                    );
                    if !valid {
                        error!("Invalid alignment in query {} to target {}", query_index, target_alignment.index);
                        error!("TargetAlignments: {:?}", target_alignment);
                        panic!("Invalid alignment");
                    }
                }
            }

            query_index += 1;
        }
        
        // Print elapsed time
        {
            info!("# Elapsed time");
            let current_mean: std::time::Duration = current_times.iter().sum::<std::time::Duration>() / current_times.len() as u32;
            let current_min = current_times.iter().min().unwrap();
            let current_max = current_times.iter().max().unwrap();
            info!("  - current: mean: {:?}, min: {:?}, max: {:?}", current_mean, current_min, current_max);
        }
    }
}
