use super::{
    DpMatrix,
    parse_valid_semi_global_result_from_dpm,
    target_indices_having_matched_pattern,
};
use sigalign::{
    results::{
        Alignment, AlignmentOperation, QueryAlignment, TargetAlignment
    },
    Reference,
    };
use sigalign_utils::sequence_reader::{
    fasta::FastaReader, SeqRecord,
};
use std::path::PathBuf;
use ahash::AHashSet;
const PREC_SCALE: u32 = 100_000;

pub fn dp_local_with_all_subs_to_pattern_existing_targets(
    query: &[u8],
    sig_reference: &Reference,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> QueryAlignment {
    let mut target_alignment_results = Vec::new();
    // Fetch target indices
    let target_indices = target_indices_having_matched_pattern(
        query,
        sig_reference,
        mismatch_penalty, 
        gap_open_penalty,
        gap_extend_penalty,
        min_length,
        max_penalty_per_length,
    );
    // Align
    for target_index in target_indices {
        let target = sig_reference.get_sequence(target_index).unwrap();

        let alignments = dp_local_with_all_subs_to_target(
            query,
            &target,
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            min_length,
            max_penalty_per_length,
        );

        if alignments.len() != 0 {
            target_alignment_results.push(TargetAlignment {
                index: target_index,
                alignments: alignments,
            });
        }
    }

    QueryAlignment(target_alignment_results)
}

pub fn dp_local_with_all_subs_to_ref_file(
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

    let mut result = Vec::new();

    while let Some(mut record) = ref_reader.next() {
        target_buffer.clear();
        record.extend_seq_buf(&mut target_buffer);

        let alignments = dp_local_with_all_subs_to_target(
            query,
            &target_buffer,
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            min_length,
            max_penalty_per_length,
        );

        if alignments.len() != 0 {
            result.push(TargetAlignment {
                index: target_index,
                alignments,
            });
        }

        target_index += 1;
    }
    QueryAlignment(result)
}

pub fn dp_local_with_all_subs_to_target(
    query: &[u8],
    target: &[u8],
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    min_length: u32,
    max_penalty_per_length: f32,
) -> Vec<Alignment> {
    let mut alignments = Vec::new();
    // Get alignment results
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
            
            let mut alignments_for_substring = parse_valid_semi_global_result_from_dpm(
                &dp_matrix, min_length, max_penalty_per_length,
            );

            adjust_position_of_alignments(
                &mut alignments_for_substring,
                query_start_index,
            );

            alignments.extend(alignments_for_substring);
        }
    };

    // Sort by
    //  1. query length - descending
    //  2. query start index - ascending
    //  3. penalty - ascending
    alignments.sort_by(|a, b| {
        let qlen1 = a.position.query.1 - a.position.query.0;
        let qlen2 = b.position.query.1 - b.position.query.0;
        qlen2.cmp(&qlen1)
            .then(a.position.query.0.cmp(&b.position.query.0))
            .then(a.penalty.cmp(&b.penalty))
    });

    // Deduplicates
    let mut paths: AHashSet<(u32, u32)> = AHashSet::new();
    let mut dedup_alignments = Vec::new();

    alignments.into_iter().for_each(|x| {
        let length = x.length;
        let penalty = x.penalty;
        if (
            length >= min_length
        ) && (
            penalty * PREC_SCALE <= (length * (max_penalty_per_length * PREC_SCALE as f32) as u32)
        ) {
            let current_paths: AHashSet<(u32, u32)> = get_alignment_paths(&x);

            if paths.is_disjoint(&current_paths) {
                dedup_alignments.push(x);
            }
            paths.extend(&current_paths);
        }
    });

    dedup_alignments
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
fn get_alignment_paths(alignment: &Alignment) -> AHashSet<(u32, u32)> {
    let (mut query_index, mut target_index) = {
        let query_index = alignment.position.query.0;
        let target_index = alignment.position.target.0;
        (query_index, target_index)
    };
    let mut paths = AHashSet::new();
    alignment.operations.iter().for_each(|operation| {
        match operation.operation {
            AlignmentOperation::Match | AlignmentOperation::Subst => {
                for _ in 0..operation.count {
                    paths.insert((query_index, target_index));
                    query_index += 1;
                    target_index += 1;
                }
            },
            AlignmentOperation::Insertion => {
                query_index += operation.count;
            },
            AlignmentOperation::Deletion => {
                target_index += operation.count;
            },
        }
    });
    paths
}