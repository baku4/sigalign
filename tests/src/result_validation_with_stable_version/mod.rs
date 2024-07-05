use std::cmp::Ordering;
use log::{error, info};

use crate::common::{init_logger, test_data_path::{
    get_qry_for_val_path, get_ref_for_val_path,
}};

mod result_converter_of_v03;
use result_converter_of_v03::stable_result_to_current_result;

mod check_results;
use check_results::is_acceptable_query_alignment;

// Test options:
// (mismatch_penalty, gap_open_penalty, gap_extend_penalty, min_length, max_penalty_per_length)
const ALIGNER_OPTIONS: [
    (u32, u32, u32, u32, f32); 2
] = [
    (4, 6, 2, 100, 0.1),
    (3, 5, 1, 80, 0.09)
];

use sigalign_utils::sequence_reader::{
    SeqRecord,
    SeqRefRecord,
    IdRecord,
    IdRefRecord,
    fasta::{FastaReader, FastaRecord},
    fastq::{FastqReader, FastqRecord},
};
use sigalign::{
    Aligner as CurrentAligner,
    Reference as CurrentReference,
    ReferenceBuilder,
    algorithms::Local,
    results::{Alignment, QueryAlignment, TargetAlignment},
};
use sigalign_stable::{
    results::AlignmentResult as StableQueryAlignment,
    wrapper::{
        DefaultAligner as StableAligner,
        DefaultReference as StableReference,
    },
};

#[test]
fn test_local_mode_of_current_algorithm() {
    init_logger();

    // Prepare paths
    let ref_file = get_ref_for_val_path();
    let qry_file = get_qry_for_val_path();

    // Prepare reference
    info!("Start to prepare reference");
    let current_reference = ReferenceBuilder::new()
        .add_fasta_file(&ref_file).unwrap()
        .build().unwrap();
    let stable_reference = StableReference::from_fasta_file(
        &ref_file
    ).unwrap();

    // Start to compare
    for aligner_option in ALIGNER_OPTIONS.iter() {
        // Time to compare
        let mut current_times = Vec::new();
        let mut stable_times = Vec::new();
        info!("Start to compare with aligner option: {:?}", aligner_option);
        let mut current_aligner = CurrentAligner::new(
            Local::new(
                aligner_option.0,
                aligner_option.1,
                aligner_option.2,
                aligner_option.3,
                aligner_option.4,
            ).unwrap()
        );

        let mut stable_aligner = StableAligner::new_local(
            aligner_option.0,
            aligner_option.1,
            aligner_option.2,
            aligner_option.3,
            aligner_option.4,
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

            // Current
            let current_result = {
                let start = std::time::Instant::now();
                let result = current_aligner.align(&query_buffer, &current_reference);
                let duration = start.elapsed();
                current_times.push(duration);
                result
            };

            let stable_result = {
                let start = std::time::Instant::now();
                let result = stable_aligner.align_query(&stable_reference, &query_buffer);
                let duration = start.elapsed();
                stable_times.push(duration);
                result
            };
            let stable_result = stable_result_to_current_result(stable_result);

            if !is_acceptable_query_alignment(&current_result, &stable_result) {
                error!("Query index {}", query_index);
            }
            query_index += 1;
        }
        
        // Print elapsed time
        {
            let current_mean: std::time::Duration = current_times.iter().sum::<std::time::Duration>() / current_times.len() as u32;
            let current_min = current_times.iter().min().unwrap();
            let current_max = current_times.iter().max().unwrap();

            let stable_mean: std::time::Duration = stable_times.iter().sum::<std::time::Duration>() / stable_times.len() as u32;
            let stable_min = stable_times.iter().min().unwrap();
            let stable_max = stable_times.iter().max().unwrap();
            info!("# Elapsed time");
            info!("  - current: mean: {:?}, min: {:?}, max: {:?}", current_mean, current_min, current_max);
            info!("  - stable: mean: {:?}, min: {:?}, max: {:?}", stable_mean, stable_min, stable_max);
        }
    }
}
