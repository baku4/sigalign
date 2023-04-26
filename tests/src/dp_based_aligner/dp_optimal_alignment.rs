use sigalign::results::{
    AlignmentPosition,
    AlignmentOperations,
    AlignmentOperation,
    AnchorAlignmentResult,
};
use super::PREC_SCALE;

use bio::alignment::pairwise::*;
use bio::alignment::pairwise::Aligner as AlignerFromCrateBio;
use bio::alignment::Alignment as AlignmentFromCrateBio;
use bio::alignment::AlignmentOperation as AlignmentOperationFromCrateBio;

pub fn optimal_semi_global_alignment(
    record: &[u8],
    query: &[u8],
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    maximum_penalty_per_scale: usize,
) -> Option<AnchorAlignmentResult> {
    // Sequence to align
    let left_record = &record[..record_start_position];
    let left_query = &query[..query_start_position];

    let right_record = &record[record_start_position+pattern_size..];
    let right_query = &query[query_start_position+pattern_size..];

    // Scoring matrix
    let scoring = Scoring::from_scores(
        gap_open_penalty as i32 * -1,
        gap_extend_penalty as i32 * -1,
        0,
        mismatch_penalty as i32 * -1,
    ).xclip_prefix(MIN_SCORE).yclip_prefix(MIN_SCORE).xclip_suffix(MIN_SCORE).yclip_suffix(MIN_SCORE);

    let left_scoring = if left_record.len() > left_query.len() {
        scoring.clone().xclip_prefix(0)
    } else {
        scoring.clone().yclip_prefix(0)
    };
    let right_scoring = if right_record.len() > right_query.len() {
        scoring.clone().xclip_suffix(0)
    } else {
        scoring.clone().yclip_suffix(0)
    };
   
    // Alignment
    let mut left_aligner = AlignerFromCrateBio::with_scoring(left_scoring);
    let left_alignment = left_aligner.custom(left_record, left_query);

    let mut right_aligner = AlignerFromCrateBio::with_scoring(right_scoring);
    let right_alignment = right_aligner.custom(right_record, right_query);

    // Position & Operation
    let (position, operations) = merge_left_and_right_alignments(
        &left_alignment,
        &right_alignment,
        record_start_position as u32,
        query_start_position as u32,
        pattern_size as u32,
    );

    // Cutoff Check
    let length: u32 = operations.iter()
        .map(|AlignmentOperations { operation: _, count }| *count)
        .sum();
    let length = length as usize;
    
    let penalty = ((left_alignment.score + right_alignment.score) * -1) as usize;

    let penalty_per_scale_of_this_alignment = PREC_SCALE as usize * penalty / length;

    if (length >= minimum_aligned_length) && (penalty_per_scale_of_this_alignment <= maximum_penalty_per_scale) {
        Some(AnchorAlignmentResult {
            penalty: penalty as u32,
            length: length as u32,
            position,
            operations,
        })
    } else {
        None
    }
}

pub fn optimal_local_alignment(
    record: &[u8],
    query: &[u8],
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    maximum_penalty_per_scale: usize,
) -> Option<AnchorAlignmentResult> {
    // Sequence to align
    let left_record = &record[..record_start_position];
    let left_query = &query[..query_start_position];

    let right_record = &record[record_start_position+pattern_size..];
    let right_query = &query[query_start_position+pattern_size..];

    // Get substring sorted by length
    let left_query_len = left_query.len();
    let left_substrings: Vec<&[u8]> = (0..left_query_len).map(|left_query_start_index| {
        &left_query[left_query_start_index..left_query_len]
    }).collect();

    let right_query_len = right_query.len();
    let right_substrings: Vec<&[u8]> = (1..=right_query_len).map(|right_query_end_index| {
        &right_query[..right_query_end_index]
    }).rev().collect();

    // Scoring matrix
    let scoring = Scoring::from_scores(
        gap_open_penalty as i32 * -1,
        gap_extend_penalty as i32 * -1,
        0,
        mismatch_penalty as i32 * -1,
    ).xclip_prefix(MIN_SCORE).yclip_prefix(MIN_SCORE).xclip_suffix(MIN_SCORE).yclip_suffix(MIN_SCORE);

    // Get Alignments of all substring
    //  - Left
    let mut left_alignments_with_length_and_penalty: Vec<(AlignmentFromCrateBio, usize, usize)> = Vec::with_capacity(left_substrings.len()+1);
    left_alignments_with_length_and_penalty.push((AlignmentFromCrateBio::default(), 0, 0));
    for left_substring in left_substrings {
        let left_scoring = if left_record.len() > left_substring.len() {
            scoring.clone().xclip_prefix(0)
        } else {
            scoring.clone().yclip_prefix(0)
        };
        let mut left_aligner = AlignerFromCrateBio::with_scoring(left_scoring);
        let left_alignment = left_aligner.custom(left_record, left_substring);
        let (length, penalty) = get_length_and_penalty_of_alignment(&left_alignment);

        left_alignments_with_length_and_penalty.push((left_alignment, length, penalty));

        let satisfy_cutoff_alone = (length >= minimum_aligned_length) && (penalty * PREC_SCALE as usize <= maximum_penalty_per_scale * length);
        if satisfy_cutoff_alone {
            break
        }
    }
    
    //  - Right
    let mut right_alignments_with_length_and_penalty: Vec<(AlignmentFromCrateBio, usize, usize)> = Vec::with_capacity(right_substrings.len()+1);
    right_alignments_with_length_and_penalty.push((AlignmentFromCrateBio::default(), 0, 0));
    for right_substring in right_substrings {
        let right_scoring = if right_record.len() > right_query.len() {
            scoring.clone().xclip_suffix(0)
        } else {
            scoring.clone().yclip_suffix(0)
        };
        let mut right_aligner = AlignerFromCrateBio::with_scoring(right_scoring);
        let right_alignment = right_aligner.custom(right_record, right_substring);
        let (length, penalty) = get_length_and_penalty_of_alignment(&right_alignment);
        right_alignments_with_length_and_penalty.push((right_alignment, length, penalty));

        let satisfy_cutoff_alone = (length >= minimum_aligned_length) && (penalty * PREC_SCALE as usize <= maximum_penalty_per_scale * length);
        if satisfy_cutoff_alone {
            break
        }
    }

    // Find best position
    let mut indices_of_alignment: Vec<IndexOfAlignment> = Vec::new();
    for (left_index, (left_alignment, left_len, left_pen)) in left_alignments_with_length_and_penalty.iter().enumerate() {
        for (right_index, (right_alignment, right_len, right_pen)) in right_alignments_with_length_and_penalty.iter().enumerate() {
            let length = left_len + right_len + pattern_size;
            let query_length = get_query_length(left_alignment) + get_query_length(right_alignment) + pattern_size;
            let penalty = left_pen + right_pen;
            indices_of_alignment.push(
                IndexOfAlignment {
                    left_alignment_index: left_index,
                    right_alignment_index: right_index,
                    length,
                    query_length,
                    penalty,
                }
            )
        }
    }
    // Sort by:
    //  - (1) Longer query length is left
    //  - (2) smaller penalty is left
    indices_of_alignment.sort_by(|a, b| {
        if a.query_length == b.query_length {
            a.penalty.partial_cmp(&b.penalty).unwrap()
        } else {
            b.length.partial_cmp(&a.length).unwrap()
        }
    });

    let optimal_index_of_alignment = {
        let mut optimal_index_of_alignment = None;
        for index_of_alignment in indices_of_alignment {
            let satisfy_cutoff = (index_of_alignment.length >= minimum_aligned_length) && (index_of_alignment.penalty * PREC_SCALE as usize <= maximum_penalty_per_scale * index_of_alignment.length);

            if satisfy_cutoff {
                optimal_index_of_alignment = Some(index_of_alignment);
                break
            }
        }
        optimal_index_of_alignment
    };
    
    match optimal_index_of_alignment {
        Some(index_of_alignment) => {
            let (left_alignment, _, _) = &left_alignments_with_length_and_penalty[index_of_alignment.left_alignment_index];
            let (right_alignment, _, _) = &right_alignments_with_length_and_penalty[index_of_alignment.right_alignment_index];

            let (position, operations) = merge_left_and_right_alignments(
                &left_alignment,
                &right_alignment,
                record_start_position as u32,
                query_start_position as u32,
                pattern_size as u32,
            );
            Some(AnchorAlignmentResult {
                penalty: index_of_alignment.penalty as u32,
                length: index_of_alignment.length as u32,
                position,
                operations,
            })
        },
        None => {
            None
        },
    }
}

fn merge_left_and_right_alignments(
    left_alignment: &AlignmentFromCrateBio,
    right_alignment: &AlignmentFromCrateBio,
    record_start_position: u32,
    query_start_position: u32,
    pattern_size: u32,
) -> (AlignmentPosition, Vec<AlignmentOperations>) {
    // Position & Operation
    let position = AlignmentPosition {
        target: (
            record_start_position - left_alignment.x_aln_len() as u32,
            record_start_position + pattern_size + right_alignment.x_aln_len() as u32,
        ),
        query: (
            query_start_position - left_alignment.y_aln_len() as u32,
            query_start_position + pattern_size + right_alignment.y_aln_len() as u32,
        ),
    };

    let mut operations: Vec<AlignmentOperations> = Vec::new();

    left_alignment.operations.iter().for_each(|operation| {
        add_one_operation(&mut operations, operation);
    });

    for _ in 0..pattern_size {
        add_one_operation(
            &mut operations,
            &AlignmentOperationFromCrateBio::Match
        );
    }

    right_alignment.operations.iter().for_each(|operation| {
        add_one_operation(&mut operations, operation);
    });

    (position, operations)
}
fn add_one_operation(
    alignment_operations: &mut Vec<AlignmentOperations>,
    alignment_type_from_crate_bio: &AlignmentOperationFromCrateBio,
) {
    let alignment_type_to_add = match alignment_type_from_crate_bio {
        AlignmentOperationFromCrateBio::Match => AlignmentOperation::Match,
        AlignmentOperationFromCrateBio::Subst => AlignmentOperation::Subst,
        AlignmentOperationFromCrateBio::Ins => AlignmentOperation::Insertion,
        AlignmentOperationFromCrateBio::Del => AlignmentOperation::Deletion,
        _ => return
    };

    if let Some(alignment_operation) = alignment_operations.last_mut() {
        if alignment_type_to_add == alignment_operation.operation {
            alignment_operation.count += 1;
            return
        }
    }

    alignment_operations.push(
        AlignmentOperations {
            operation: alignment_type_to_add,
            count: 1,
        }
    );
}


#[derive(Debug)]
struct IndexOfAlignment {
    left_alignment_index: usize,
    right_alignment_index: usize,
    length: usize,
    query_length: usize,
    penalty: usize,
}
fn get_query_length(alignment: &AlignmentFromCrateBio) -> usize {
    alignment.y_aln_len()
}
fn get_length_and_penalty_of_alignment(alignment: &AlignmentFromCrateBio) -> (usize, usize) {
    let length: usize = alignment.operations.iter().map(|op| {
        match op {
            AlignmentOperationFromCrateBio::Match
            | AlignmentOperationFromCrateBio::Subst
            | AlignmentOperationFromCrateBio::Del
            | AlignmentOperationFromCrateBio::Ins => {
                1
            },
            _ => {
                0
            },
        }
    }).sum();
    let penalty = (alignment.score * -1) as usize;
    (length, penalty)
}

#[cfg(feature = "print")]
#[test]
fn print_sorted_indices_of_alignment() {
    use rand::prelude::*;

    let mut rng = rand::thread_rng();

    let index_count = 10;

    let raw_vec: Vec<usize> = (0..index_count).collect();

    let mut length = raw_vec.clone();
    let mut query_length = raw_vec.clone();
    for i in index_count / 2..index_count {
        query_length[i] = query_length[i] / 2;
    }
    query_length.shuffle(&mut rng);
    let mut penalty = raw_vec.clone();
    penalty.shuffle(&mut rng);
    
    let mut ioa_list = Vec::with_capacity(index_count);

    for ((v1, v2), v3) in length.iter().zip(query_length.iter()).zip(penalty.iter()) {
        ioa_list.push(
            IndexOfAlignment {
                left_alignment_index: 0,
                right_alignment_index: 0,
                length: *v1,
                query_length: *v2,
                penalty: *v3,
            }
        )
    }

    ioa_list.sort_by(|a, b| {
        if a.query_length == b.query_length {
            a.penalty.partial_cmp(&b.penalty).unwrap()
        } else {
            b.length.partial_cmp(&a.length).unwrap()
        }
    });

    println!("{:#?}", ioa_list);
}
