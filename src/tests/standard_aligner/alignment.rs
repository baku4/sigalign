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
    
    let left_semi_global_penalty = (left_alignment.score * -1) as usize;
    let left_semi_global_length = left_alignment.operations.iter().filter(|&x| {
        match x {
            AlignmentOperationFromCrateBio::Xclip(_) => false,
            AlignmentOperationFromCrateBio::Yclip(_) => false,
            _ => true,
        }
    }).count();

    let mut left_operations: Vec<AlignmentOperation> = Vec::new();
    left_alignment.operations.iter().for_each(|operation| {
        add_one_operation(
            &mut left_operations,
            operation
        );
    });
    // Add first
    left_points_of_local.push(LeftPointOfLocalAlignment {
        penalty: left_semi_global_penalty,
        length: left_semi_global_length,
        start_index_of_operations: 0,
    });

    let mut left_accumulated_penalty = 0;
    let mut left_accumulated_length = 0;
    left_operations.iter().enumerate().for_each(|(
        start_index, AlignmentOperation { alignment_type, count }
    )| {
        match alignment_type {
            AlignmentType::Match => {
                left_accumulated_length += *count as usize;
            },
            AlignmentType::Subst => {
                left_points_of_local.push(LeftPointOfLocalAlignment {
                    penalty: left_semi_global_penalty - left_accumulated_penalty,
                    length: left_semi_global_length - left_accumulated_length,
                    start_index_of_operations: start_index,
                });

                for _ in 0..*count {
                    left_accumulated_penalty += mismatch_penalty;
                    left_accumulated_length += 1;

                    left_points_of_local.push(LeftPointOfLocalAlignment {
                        penalty: left_semi_global_penalty - left_accumulated_penalty,
                        length: left_semi_global_length - left_accumulated_length,
                        start_index_of_operations: start_index,
                    });
                }
            },
            AlignmentType::Insertion | AlignmentType::Deletion => {
                left_points_of_local.push(LeftPointOfLocalAlignment {
                    penalty: left_semi_global_penalty - left_accumulated_penalty,
                    length: left_semi_global_length - left_accumulated_length,
                    start_index_of_operations: start_index,
                });

                left_accumulated_penalty += gap_open_penalty;
                for _ in 0..*count {
                    left_accumulated_penalty += gap_extend_penalty;
                    left_accumulated_length += 1;

                    left_points_of_local.push(LeftPointOfLocalAlignment {
                        penalty: left_semi_global_penalty - left_accumulated_penalty,
                        length: left_semi_global_length - left_accumulated_length,
                        start_index_of_operations: start_index,
                    });
                }
            },
        }
    });

    // (2) Right
    let mut right_points_of_local: Vec<RightPointOfLocalAlignment> = Vec::new();

    let mut right_operations: Vec<AlignmentOperation> = Vec::new();
    right_alignment.operations.iter().for_each(|operation| {
        add_one_operation(
            &mut right_operations,
            operation
        );
    });

    let mut right_accumulated_penalty = 0;
    let mut right_accumulated_length = 0;
    right_operations.iter().enumerate().for_each(|(
        end_index, AlignmentOperation { alignment_type, count }
    )| {
        match alignment_type {
            AlignmentType::Match => {
                right_accumulated_length += *count as usize;
            },
            AlignmentType::Subst => {
                right_points_of_local.push(RightPointOfLocalAlignment {
                    penalty: right_accumulated_penalty,
                    length: right_accumulated_length,
                    end_index_of_operations: end_index,
                });

                for _ in 0..*count {
                    right_accumulated_penalty += mismatch_penalty;
                    right_accumulated_length += 1;

                    right_points_of_local.push(RightPointOfLocalAlignment {
                        penalty: right_accumulated_penalty,
                        length: right_accumulated_length,
                        end_index_of_operations: end_index,
                    });
                }
            },
            AlignmentType::Insertion | AlignmentType::Deletion  => {
                right_points_of_local.push(RightPointOfLocalAlignment {
                    penalty: right_accumulated_penalty,
                    length: right_accumulated_length,
                    end_index_of_operations: end_index,
                });

                right_accumulated_penalty += gap_open_penalty;
                for _ in 0..*count {
                    right_accumulated_penalty += gap_extend_penalty;
                    right_accumulated_length += 1;

                    right_points_of_local.push(RightPointOfLocalAlignment {
                        penalty: right_accumulated_penalty,
                        length: right_accumulated_length,
                        end_index_of_operations: end_index,
                    });
                }
            },
        }
    });
    // Add last
    let right_semi_global_penalty = (right_alignment.score * -1) as usize;
    let right_semi_global_length = right_alignment.operations.iter().filter(|&x| {
        match x {
            AlignmentOperationFromCrateBio::Xclip(_) => false,
            AlignmentOperationFromCrateBio::Yclip(_) => false,
            _ => true,
        }
    }).count();

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

    let best_position = match best_position {
        Some(best_position) => best_position,
        None => return None
    };

    // println!("best_position:\n{:?}", best_position);

    // Get operations
    let operations = {
        let mut operations: Vec<AlignmentOperation> = Vec::new();
        operations.extend_from_slice(&left_operations[best_position.0.start_index_of_operations..]);

        if let Some(AlignmentOperation {
            alignment_type: AlignmentType::Match,
            count,
        }) = operations.last_mut() {
            *count += pattern_size as u32;
        } else {
            operations.push(
                AlignmentOperation {
                    alignment_type: AlignmentType::Match,
                    count: pattern_size as u32,
                }
            )
        }

        if let Some(AlignmentOperation {
            alignment_type: AlignmentType::Match,
            count,
        }) = right_operations.first() {
            {
                let last_operation = operations.last_mut().unwrap();
                last_operation.count += *count;
            }
            operations.extend_from_slice(
                &right_operations[1..best_position.1.end_index_of_operations]
            );
        } else {
            operations.extend_from_slice(
                &right_operations[..best_position.1.end_index_of_operations]
            );
        }

        operations
    };

    // Get AlignmentPosition
    let mut left_inbound_insertion_count = 0;
    let mut left_inbound_deletion_count = 0;
    let mut right_inbound_insertion_count = 0;
    let mut right_inbound_deletion_count = 0;

    left_operations[best_position.0.start_index_of_operations..].iter().for_each(|AlignmentOperation { alignment_type, count }| {
        match alignment_type {
            AlignmentType::Insertion => {
                left_inbound_insertion_count += *count as usize;
            },
            AlignmentType::Deletion => {
                left_inbound_deletion_count += *count as usize;
            },
            _ => (),
        }
    });

    right_operations[..best_position.1.end_index_of_operations].iter().for_each(|AlignmentOperation { alignment_type, count }| {
        match alignment_type {
            AlignmentType::Insertion => {
                right_inbound_insertion_count += *count as usize;
            },
            AlignmentType::Deletion => {
                right_inbound_deletion_count += *count as usize;
            },
            _ => (),
        }
    });

    let position = AlignmentPosition {
        record: (
            record_start_position - best_position.0.length + left_inbound_deletion_count,
            record_start_position + pattern_size + best_position.1.length - right_inbound_deletion_count
        ),
        query: (
            query_start_position - best_position.0.length + left_inbound_insertion_count,
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
