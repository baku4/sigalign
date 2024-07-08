use super::{
    DpMatrix,
    calculate_the_pattern_size,
    target_indices_having_matched_pattern,
};
use sigalign::{
    results::{
        QueryAlignment,
        TargetAlignment,
        Alignment,
        AlignmentOperation,
    },
    Aligner,
    Reference,
    ReferenceBuilder,
};
use sigalign_utils::sequence_reader::{
    fasta::FastaReader,
    SeqRecord,
};
use ahash::AHashSet;
use std::path::PathBuf;

pub fn local_all_substring_with_dpm_only_to_pattern_matched_targets(
    query: &[u8],
    sig_reference: &Reference,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> QueryAlignment {
    // Init
    let mut target_alignment_results = Vec::new();
    // Cal pattern size
    let pattern_size = calculate_the_pattern_size(
        mismatch_penalty, 
        gap_open_penalty,
        gap_extend_penalty,
        min_length,
        max_penalty_per_length,
    ).unwrap();
    // Fetch target indices
    let target_indices = target_indices_having_matched_pattern(
        query,
        sig_reference,
        pattern_size,
    );
    // Align
    for target_index in target_indices {
        let target = sig_reference.get_sequence(target_index).unwrap();

        // Get anchor alignment results
        let mut all_anchor_alignment_results = Vec::new();
        let query_length = query.len();
        for substring_length in (1..=query_length).rev() {
            for query_start_index in 0..(query_length+1-substring_length) {
                let query_last_index = query_start_index + substring_length;
                let substring = query[query_start_index..query_last_index].to_vec();
                let dp_matrix = DpMatrix::new(
                    substring,
                    target.to_vec(),
                    mismatch_penalty,
                    gap_open_penalty,
                    gap_extend_penalty,
                );
                
                let mut anchor_alignment_results = dp_matrix.parse_valid_semi_global_result(min_length, max_penalty_per_length);

                adjust_position_of_alignments(
                    &mut anchor_alignment_results,
                    query_start_index,
                );

                all_anchor_alignment_results.extend(anchor_alignment_results);
            }
        };

        // Sort by
        //  1. query length - descending
        //  2. query start index - ascending
        all_anchor_alignment_results.sort_by(|a, b| {
            let qlen1 = a.position.query.1 - a.position.query.0;
            let qlen2 = b.position.query.1 - b.position.query.0;
            qlen2.cmp(&qlen1)
                .then(a.position.query.0.cmp(&b.position.query.0))
        });

        // Deduplicates
        let mut target_alignment_result: TargetAlignment = TargetAlignment {
            index: target_index,
            alignments: Vec::new(),
        };
        let mut paths: AHashSet<(u32, u32)> = AHashSet::new();
        all_anchor_alignment_results.into_iter().for_each(|x| {
            let current_paths: AHashSet<(u32, u32)> = get_alignment_paths(&x);

            if paths.is_disjoint(&current_paths) {
                paths.extend(&current_paths);
                target_alignment_result.alignments.push(x);
            }
        });

        if target_alignment_result.alignments.len() != 0 {
            target_alignment_results.push(target_alignment_result)
        }
    }

    QueryAlignment(target_alignment_results)
}

pub fn local_all_substring_with_dpm_using_all_target(
    query: &[u8],
    ref_file: &PathBuf,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> QueryAlignment {
    let mut ref_reader = FastaReader::from_path(ref_file).unwrap();
    let mut target_buffer = Vec::new();
    let mut target_index = 0;
    let mut target_alignment_results = Vec::new();

    while let Some(mut record) = ref_reader.next() {
        target_buffer.clear();
        record.extend_seq_buf(&mut target_buffer);

        let mut target_alignment_result: TargetAlignment = TargetAlignment {
            index: target_index,
            alignments: Vec::new(),
        };

        let mut paths: AHashSet<(u32, u32)> = AHashSet::new();

        let query_length = query.len();
        for substring_length in (1..=query_length).rev() {
            for query_start_index in 0..(query_length+1-substring_length) {
                let query_last_index = query_start_index + substring_length;
                let substring = query[query_start_index..query_last_index].to_vec();
                let dp_matrix = DpMatrix::new(
                    substring,
                    target_buffer.clone(),
                    mismatch_penalty,
                    gap_open_penalty,
                    gap_extend_penalty,
                );

                let mut anchor_alignment_results = dp_matrix.parse_valid_semi_global_result(min_length, max_penalty_per_length);

                adjust_position_of_alignments(
                    &mut anchor_alignment_results,
                    query_start_index,
                );

                for anchor_alignment_result in anchor_alignment_results {
                    let current_paths: AHashSet<(u32, u32)> = get_alignment_paths(&anchor_alignment_result);

                    if paths.is_disjoint(&current_paths) {
                        paths.extend(&current_paths);
                        target_alignment_result.alignments.push(anchor_alignment_result);
                    }
                }
            }
        };

        if target_alignment_result.alignments.len() != 0 {
            target_alignment_results.push(target_alignment_result)
        }

        target_index += 1;
    }

    QueryAlignment(target_alignment_results)
}

fn adjust_position_of_alignments(
    alignments: &mut Vec<Alignment>,
    query_start_index: usize,
) {
    alignments.iter_mut().for_each(|result| {
        let query_position = &mut result.position.query;
        query_position.0 += query_start_index as u32;
        query_position.1 += query_start_index as u32;
    });
}
fn get_alignment_paths(
    alignments: &Alignment,
) -> AHashSet<(u32, u32)> {
    let (mut query_index, mut target_index) = {
        let query_index = alignments.position.query.0;
        let target_index = alignments.position.target.0;
        (query_index, target_index)
    };
    let mut paths = AHashSet::new();
    alignments.operations.iter().for_each(|operation| {
        match operation.operation {
            AlignmentOperation::Match | AlignmentOperation::Subst => {
                for _ in 0..operation.count {
                    paths.insert((query_index, target_index));
                    query_index += 1;
                    target_index += 1;
                }
            },
            AlignmentOperation::Insertion => {
                target_index += operation.count;
            },
            AlignmentOperation::Deletion => {
                query_index += operation.count;
            },
        }
    });
    paths
}