use super::*;

use bio::alignment::pairwise::*;
use bio::alignment::pairwise::Aligner as AlignerFromCrateBio;
use bio::alignment::AlignmentOperation as AlignmentOperationFromCrateBio;

pub fn semi_global_alignment_with_position(
    record: Sequence,
    query: Sequence,
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    penalty_per_million: usize,
) -> Option<AlignmentResult> {
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
    let position = AlignmentPosition {
        record: (
            left_alignment.xstart,
            record_start_position + pattern_size + right_alignment.xend
        ),
        query: (
            left_alignment.ystart,
            query_start_position + pattern_size + right_alignment.yend
        ),
    };

    let mut operations: Vec<AlignmentOperation> = Vec::new();

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

    // Cutoff Check
    let length: u32 = operations.iter()
        .map(|AlignmentOperation { alignment_type: _, count }| *count)
        .sum();
    let length = length as usize;
    
    let penalty = ((left_alignment.score + right_alignment.score) * -1) as usize;

    let penalty_per_million_of_this_alignment = 1_000_000 * penalty / length;

    if (length >= minimum_aligned_length) && (penalty_per_million_of_this_alignment <= penalty_per_million) {
        Some(AlignmentResult {
            dissimilarity: penalty_per_million_of_this_alignment as f32 / 1_000_000.0,
            penalty,
            length,
            position,
            operations,
        })
    } else {
        None
    }
}

fn add_one_operation(
    alignment_operations: &mut Vec<AlignmentOperation>,
    alignment_type_from_crate_bio: &AlignmentOperationFromCrateBio,
) {
    let alignment_type_to_add = match alignment_type_from_crate_bio {
        AlignmentOperationFromCrateBio::Match => AlignmentType::Match,
        AlignmentOperationFromCrateBio::Subst => AlignmentType::Subst,
        AlignmentOperationFromCrateBio::Ins => AlignmentType::Insertion,
        AlignmentOperationFromCrateBio::Del => AlignmentType::Deletion,
        _ => return
    };

    if let Some(alignment_operation) = alignment_operations.last_mut() {
        if alignment_type_to_add == alignment_operation.alignment_type {
            alignment_operation.count += 1;
            return
        }
    }

    alignment_operations.push(
        AlignmentOperation {
            alignment_type: alignment_type_to_add,
            count: 1,
        }
    );
}