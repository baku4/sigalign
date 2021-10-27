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
    penalty_per_scale: usize,
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

    let penalty_per_scale_of_this_alignment = PRECISION_SCALE * penalty / length;

    if (length >= minimum_aligned_length) && (penalty_per_scale_of_this_alignment <= penalty_per_scale) {
        Some(AlignmentResult {
            dissimilarity: penalty_per_scale_of_this_alignment as f32 / PRECISION_SCALE as f32,
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

pub fn local_alignment_with_position(
    record: Sequence,
    query: Sequence,
    record_start_position: usize,
    query_start_position: usize,
    pattern_size: usize,
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    penalty_per_scale: usize,
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

    //
    //
    // Get operations and position candidates
    // 
    //  - Best position: alignment with longest length satisfying cutoff
    //
    //
    // (1) Left
    let mut left_points_of_local: Vec<LeftPointOfLocalAlignment> = Vec::new();

    let left_operations: Vec<AlignmentOperationFromCrateBio> = left_alignment.operations.iter().filter_map(|&x| {
        match x {
            AlignmentOperationFromCrateBio::Xclip(_) | AlignmentOperationFromCrateBio::Yclip(_) => None,
            _ => Some(x),
        }
    }).collect();
    let left_semi_global_penalty = (left_alignment.score * -1) as usize;
    let left_semi_global_length = left_operations.len();

    let mut left_accumulated_penalty = 0;
    let mut left_accumulated_length = 0;

    let mut previous_alignment_operation = AlignmentOperationFromCrateBio::Match;

    left_operations.iter().enumerate().rev().for_each(|(start_index, operation_from_crate_bio)| {
        match operation_from_crate_bio {
            AlignmentOperationFromCrateBio::Match => {
                left_accumulated_length += 1;
                previous_alignment_operation = AlignmentOperationFromCrateBio::Match;
            },
            AlignmentOperationFromCrateBio::Subst => {
                left_points_of_local.push(LeftPointOfLocalAlignment {
                    penalty: left_accumulated_penalty,
                    length: left_accumulated_length,
                    start_index_of_operations: start_index,
                });

                left_accumulated_length += 1;
                left_accumulated_penalty += mismatch_penalty;
                previous_alignment_operation = AlignmentOperationFromCrateBio::Subst;
            },
            AlignmentOperationFromCrateBio::Ins => {
                left_points_of_local.push(LeftPointOfLocalAlignment {
                    penalty: left_accumulated_penalty,
                    length: left_accumulated_length,
                    start_index_of_operations: start_index,
                });

                left_accumulated_length += 1;
                if previous_alignment_operation ==  AlignmentOperationFromCrateBio::Ins {
                    left_accumulated_penalty += gap_extend_penalty;
                } else {
                    left_accumulated_penalty += gap_open_penalty + gap_extend_penalty;
                }
                previous_alignment_operation = AlignmentOperationFromCrateBio::Ins;
            },
            AlignmentOperationFromCrateBio::Del => {
                left_points_of_local.push(LeftPointOfLocalAlignment {
                    penalty: left_accumulated_penalty,
                    length: left_accumulated_length,
                    start_index_of_operations: start_index,
                });

                left_accumulated_length += 1;
                if previous_alignment_operation ==  AlignmentOperationFromCrateBio::Del {
                    left_accumulated_penalty += gap_extend_penalty;
                } else {
                    left_accumulated_penalty += gap_open_penalty + gap_extend_penalty;
                }
                previous_alignment_operation = AlignmentOperationFromCrateBio::Del;
            },
            _ => {},
        }
    });

    // Add last
    left_points_of_local.push(LeftPointOfLocalAlignment {
        penalty: left_semi_global_penalty,
        length: left_semi_global_length,
        start_index_of_operations: 0,
    });

    // (2) Right
    let mut right_points_of_local: Vec<RightPointOfLocalAlignment> = Vec::new();

    let right_operations: Vec<AlignmentOperationFromCrateBio> = right_alignment.operations.iter().filter_map(|&x| {
        match x {
            AlignmentOperationFromCrateBio::Xclip(_) | AlignmentOperationFromCrateBio::Yclip(_) => None,
            _ => Some(x),
        }
    }).collect();
    let right_semi_global_penalty = (right_alignment.score * -1) as usize;
    let right_semi_global_length = right_operations.len();

    let mut right_accumulated_penalty = 0;
    let mut right_accumulated_length = 0;

    let mut previous_alignment_operation = AlignmentOperationFromCrateBio::Match;

    right_operations.iter().enumerate().for_each(|(end_index, operation_from_crate_bio)| {
        match operation_from_crate_bio {
            AlignmentOperationFromCrateBio::Match => {
                right_accumulated_length += 1;
                previous_alignment_operation = AlignmentOperationFromCrateBio::Match;
            },
            AlignmentOperationFromCrateBio::Subst => {
                right_points_of_local.push(RightPointOfLocalAlignment {
                    penalty: right_accumulated_penalty,
                    length: right_accumulated_length,
                    end_index_of_operations: end_index,
                });

                right_accumulated_length += 1;
                right_accumulated_penalty += mismatch_penalty;
                previous_alignment_operation = AlignmentOperationFromCrateBio::Subst;
            },
            AlignmentOperationFromCrateBio::Ins => {
                right_points_of_local.push(RightPointOfLocalAlignment {
                    penalty: right_accumulated_penalty,
                    length: right_accumulated_length,
                    end_index_of_operations: end_index,
                });

                right_accumulated_length += 1;
                if previous_alignment_operation ==  AlignmentOperationFromCrateBio::Ins {
                    right_accumulated_penalty += gap_extend_penalty;
                } else {
                    right_accumulated_penalty += gap_open_penalty + gap_extend_penalty;
                }
                previous_alignment_operation = AlignmentOperationFromCrateBio::Ins;
            },
            AlignmentOperationFromCrateBio::Del => {
                right_points_of_local.push(RightPointOfLocalAlignment {
                    penalty: right_accumulated_penalty,
                    length: right_accumulated_length,
                    end_index_of_operations: end_index,
                });

                right_accumulated_length += 1;
                if previous_alignment_operation ==  AlignmentOperationFromCrateBio::Del {
                    right_accumulated_penalty += gap_extend_penalty;
                } else {
                    right_accumulated_penalty += gap_open_penalty + gap_extend_penalty;
                }
                previous_alignment_operation = AlignmentOperationFromCrateBio::Del;
            },
            _ => {},
        }
    });

    // Add last
    right_points_of_local.push(RightPointOfLocalAlignment {
        penalty: right_semi_global_penalty,
        length: right_semi_global_length,
        end_index_of_operations: right_operations.len(),
    });

    // Get best position
    let mut length_of_best_position = 0;
    let mut best_position = None;

    for left_point in left_points_of_local.iter() {
        for right_point in right_points_of_local.iter() {
            let penalty = left_point.penalty + right_point.penalty;
            let length = left_point.length + right_point.length + pattern_size;

            // Check if valid
            if length >= minimum_aligned_length
            && length_of_best_position <= length 
            && PRECISION_SCALE * penalty / length <= penalty_per_scale {
                length_of_best_position = length;
                best_position = Some((left_point, right_point));
            }
        }
    }

    // println!("left_points_of_local:\n{:?}", left_points_of_local);
    // println!("right_points_of_local:\n{:?}", right_points_of_local);
    // println!("best_position:\n{:?}", best_position);

    let best_position = match best_position {
        Some(best_position) => best_position,
        None => return None
    };

    // Get operations
    let operations = {
        let mut operations: Vec<AlignmentOperation> = Vec::new();

        // Add left
        for operation in &left_operations[best_position.0.start_index_of_operations..] {
            add_one_operation(
                &mut operations,
                operation
            );
        };

        // Add pattern sized 'Match'
        for _ in 0..pattern_size {
            add_one_operation(
                &mut operations,
                &AlignmentOperationFromCrateBio::Match,
            );
        };

        // Add right
        for operation in &right_operations[..best_position.1.end_index_of_operations] {
            add_one_operation(
                &mut operations,
                operation
            );
        };

        operations
    };

    // Get AlignmentPosition
    let mut left_inbound_insertion_count = 0;
    let mut left_inbound_deletion_count = 0;
    let mut right_inbound_insertion_count = 0;
    let mut right_inbound_deletion_count = 0;

    left_operations[best_position.0.start_index_of_operations..].iter().for_each(|x| {
        match x {
            AlignmentOperationFromCrateBio::Ins => {
                left_inbound_insertion_count += 1;
            },
            AlignmentOperationFromCrateBio::Del => {
                left_inbound_deletion_count += 1;
            },
            _ => {},
        }
    });

    right_operations[..best_position.1.end_index_of_operations].iter().for_each(|x| {
        match x {
            AlignmentOperationFromCrateBio::Ins => {
                right_inbound_insertion_count += 1;
            },
            AlignmentOperationFromCrateBio::Del => {
                right_inbound_deletion_count += 1;
            },
            _ => {},
        }
    });

    let position = AlignmentPosition {
        record: (
            record_start_position + left_inbound_deletion_count- best_position.0.length,
            record_start_position + pattern_size + best_position.1.length - right_inbound_deletion_count
        ),
        query: (
            query_start_position + left_inbound_insertion_count - best_position.0.length,
            query_start_position + pattern_size + best_position.1.length - right_inbound_insertion_count
        ),
    };

    // Return result
    let penalty = best_position.0.penalty + best_position.1.penalty;
    let length = best_position.0.length + best_position.1.length + pattern_size;

    Some(
        AlignmentResult {
            dissimilarity: penalty as f32 / length as f32,
            penalty,
            length,
            position,
            operations,
        }
    )
}

#[derive(Debug, Clone)]
struct LeftPointOfLocalAlignment {
    penalty: usize,
    length: usize,
    start_index_of_operations: usize,
}

#[derive(Debug, Clone)]
struct RightPointOfLocalAlignment {
    penalty: usize,
    length: usize,
    end_index_of_operations: usize,
}
