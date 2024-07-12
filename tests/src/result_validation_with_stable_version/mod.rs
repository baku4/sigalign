use log::{error, info};
use crate::common::{
    init_logger,
    test_data_path::DataForValidation,
    random_regulator::{gen_random_regulator, gen_random_regulator_not_errored_in_v032},
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

mod result_converter_of_v03;
use result_converter_of_v03::stable_result_to_current_result;

mod check_results;
use check_results::is_left_query_alignment_better;

mod print_results;

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
fn test_local_mode_of_current_algorithm() {
    let current_aligner_generator = |px, po, pe, minl, maxp| {
        CurrentAligner::new(
            Local::new(px, po, pe, minl, maxp).unwrap()
        )
    };
    let stable_aligner_generator = |px, po, pe, minl, maxp| {
        StableAligner::new_local(px, po, pe, minl, maxp).unwrap()
    };
    let regulators: Vec<(u32, u32, u32, u32, f32)> = (0..ALIGNER_OPTION_COUNT)
        .map(|_| gen_random_regulator_not_errored_in_v032(ASSUMED_MAX_MISMATCH_PER_100)).collect::<Vec<_>>();
    test_of_current_algorithm(
        &current_aligner_generator,
        Some(&stable_aligner_generator),
        regulators,
    );
}

#[test]
fn test_semi_global_mode_of_current_algorithm() {
    let current_aligner_generator = |px, po, pe, minl, maxp| {
        CurrentAligner::new(
            SemiGlobal::new(px, po, pe, minl, maxp).unwrap()
        )
    };
    let stable_aligner_generator = |px, po, pe, minl, maxp| {
        StableAligner::new_semi_global(px, po, pe, minl, maxp).unwrap()
    };
    let regulators: Vec<(u32, u32, u32, u32, f32)> = (0..ALIGNER_OPTION_COUNT)
        .map(|_| gen_random_regulator_not_errored_in_v032(ASSUMED_MAX_MISMATCH_PER_100)).collect::<Vec<_>>();
    test_of_current_algorithm(
        &current_aligner_generator,
        Some(&stable_aligner_generator),
        regulators,
    );
}

fn test_of_current_algorithm<F1, F2, A>(
    current_aligner_generator: &F1,
    stable_aligner_generator: Option<&F2>,
    regulators: Vec<(u32, u32, u32, u32, f32)>,
) where
    A: Algorithm,
    F1: Fn(u32, u32, u32, u32, f32) -> CurrentAligner<A>,
    F2: Fn(u32, u32, u32, u32, f32) -> StableAligner,
{
    init_logger();

    // Prepare paths
    let (ref_file, qry_file) = DataForValidation::Default.get_data_paths();

    // Prepare reference
    info!("Start to prepare reference");
    let current_reference = ReferenceBuilder::new()
        .add_fasta_file(&ref_file).unwrap()
        .build().unwrap();
    let stable_reference = if stable_aligner_generator.is_some() {
        Some(StableReference::from_fasta_file(
            &ref_file
        ).unwrap())
    } else {
        None
    };

    // Start to compare
    for aligner_option in regulators {
        // Time to compare
        let mut current_times = Vec::new();
        let mut stable_times = Vec::new();
        info!("Start to compare with aligner option: {:?}", aligner_option);
        let mut current_aligner = current_aligner_generator(
            aligner_option.0,
            aligner_option.1,
            aligner_option.2,
            aligner_option.3,
            aligner_option.4,
        );

        let mut stable_aligner = stable_aligner_generator.map(|f| {
            f(
                aligner_option.0,
                aligner_option.1,
                aligner_option.2,
                aligner_option.3,
                aligner_option.4,
            )
        });

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
            if query_index % 100 == 0 {
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

            // Stables
            if stable_aligner_generator.is_some() {
                let reference = stable_reference.as_ref().unwrap();
                let aligner = stable_aligner.as_mut().unwrap();

                let stable_result = {
                    let start = std::time::Instant::now();
                    let result = aligner.align_query(reference, &query_buffer);
                    let duration = start.elapsed();
                    stable_times.push(duration);
                    result
                };
                let stable_result = stable_result_to_current_result(stable_result);
    
                if !is_left_query_alignment_better(query_index, &current_result, &stable_result) {
                    error!("in query index: {}", query_index);
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

            if stable_aligner_generator.is_some() {
                let stable_mean: std::time::Duration = stable_times.iter().sum::<std::time::Duration>() / stable_times.len() as u32;
                let stable_min = stable_times.iter().min().unwrap();
                let stable_max = stable_times.iter().max().unwrap();
                info!("  - stable: mean: {:?}, min: {:?}, max: {:?}", stable_mean, stable_min, stable_max);
            }
        }
    }
}
