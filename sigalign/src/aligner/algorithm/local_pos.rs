use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};
use super::{Extension, AlignmentHashSet, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};

use super::{PointOfMaximumLength, StartPointOfWaveFront};


// ANCHOR

struct RecordAnchors {
    anchors_by_pattern: Vec<PatternAnchors>,
}

struct PatternAnchors {
    positions: Vec<usize>,
}

impl RecordAnchors {
    fn alignment(
        self,
        record_sequence: Sequence,
        query_sequence: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) { // -> Vec<AnchorAlignmentResult> {
        let spare_penalty_determinant = cutoff.maximum_penalty_per_scale * (pattern_size - 1);

        let record_length = record_sequence.len();
        let query_length = query_sequence.len();

        let pattern_count = self.anchors_by_pattern.len();
        let mut valid_position_table = ValidPositionTable::new(pattern_count);
        
        self.anchors_by_pattern.into_iter().enumerate().for_each(|(pattern_index, pattern_anchors)| {
            let query_start_index = pattern_index * pattern_size;

            pattern_anchors.positions.into_iter().for_each(|record_start_index| {
                // Extend right
                let right_record_slice = &record_sequence[record_start_index + pattern_size..];
                let right_query_slice = &query_sequence[query_start_index + pattern_size..];
                let right_record_slice_length = right_record_slice.len();
                let right_query_slice_length = right_query_slice.len();
                let right_spare_penalty = calculate_spare_penalty_from_determinant( // TODO: Change to usize
                    spare_penalty_determinant as i64,
                    pattern_size,
                    right_query_slice_length,
                    right_record_slice_length,
                    penalties,
                    cutoff,
                );

                right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
                let right_point_of_maximum_length = right_wave_front.point_of_maximum_length();
                
                // Extend left
                let left_record_slice = &record_sequence[..record_start_index];
                let left_query_slice = &query_sequence[..query_start_index];
                let left_record_slice_length = left_record_slice.len();
                let left_query_slice_length = left_query_slice.len();
                let spare_penalty_determinant_of_right = right_point_of_maximum_length.spare_penalty_determinant(cutoff);
                let left_spare_penalty = calculate_spare_penalty_from_determinant(
                    spare_penalty_determinant_of_right,
                    pattern_size,
                    left_query_slice_length,
                    left_record_slice_length,
                    penalties,
                    cutoff,
                );
                
                left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
                let left_point_of_maximum_length = left_wave_front.point_of_maximum_length();

                let point_of_maximum_length = PointOfMaximumLength::get_optional_start_points_of_wave_front(left_point_of_maximum_length, right_point_of_maximum_length, pattern_size, cutoff);

                let optional_extensions = match point_of_maximum_length {
                    Some(start_point_of_wave_front) => {
                        Some((
                            left_wave_front.backtrace_from_point(
                                start_point_of_wave_front.left_score,
                                start_point_of_wave_front.left_index_of_components,
                                penalties,
                            ),
                            right_wave_front.backtrace_from_point(
                                start_point_of_wave_front.right_score,
                                start_point_of_wave_front.right_index_of_components,
                                penalties,
                            ),
                        ))
                    },
                    None => {
                        None
                    }
                };

                match optional_extensions {
                    Some((left_extension, right_extension)) => {
                        let record_position_to_next_patterns = RecordPositionToNextPatterns::new(
                            &right_extension.operations,
                            record_start_index,
                            query_start_index,
                            pattern_size as u32,
                        );
                    },
                    None => {
                        
                    },
                }
            })
        })
    }
}

struct ValidPositionTable {
    valid_position_by_pattern: Vec<ValidPosition>,
}
impl ValidPositionTable {
    fn new(pattern_count: usize) -> Self {
        Self {
            valid_position_by_pattern: vec![ValidPosition(Vec::new()); pattern_count],
        }
    }
    fn add_valid_positions(
        &mut self,
        current_pattern_index: usize,
        record_position_to_next_patterns: RecordPositionToNextPatterns,
    ) {
        //
    }
}

#[derive(Debug, Clone)]
struct ValidPosition(Vec<usize>);

#[derive(Debug)]
struct RecordPositionToNextPatterns(Vec<Option<usize>>);

impl RecordPositionToNextPatterns {
    fn new(
        operations: &Vec<AlignmentOperation>,
        record_start_index: usize,
        query_start_index: usize,
        pattern_size: u32,
    ) -> Self {
        let mut record_position = record_start_index + pattern_size as usize;
        let mut query_position = query_start_index + pattern_size as usize;

        let mut residue_for_next_pattern = 0;

        let mut record_position_to_next_patterns: Vec<Option<usize>> = Vec::new();

        operations.iter().for_each(|alignment_operation| {
            let case = &alignment_operation.case;
            let count = alignment_operation.count;

            match case {
                AlignmentCase::Match => {
                    while residue_for_next_pattern < count {
                        residue_for_next_pattern += pattern_size;

                        if residue_for_next_pattern <= count {
                            record_position_to_next_patterns.push(
                                Some(record_position + residue_for_next_pattern as usize - pattern_size as usize)
                            );
                        } else {
                            record_position_to_next_patterns.push(None);
                        }
                    }

                    residue_for_next_pattern -= count;
                    record_position += count as usize;
                    query_position += count as usize;
                },
                AlignmentCase::Subst => {
                    while residue_for_next_pattern < count {
                        residue_for_next_pattern += pattern_size;

                        record_position_to_next_patterns.push(None);
                    }

                    residue_for_next_pattern -= count;
                    record_position += count as usize;
                    query_position += count as usize;
                },
                AlignmentCase::Insertion => {
                    while residue_for_next_pattern < count {
                        residue_for_next_pattern += pattern_size;

                        record_position_to_next_patterns.push(None);
                    }

                    residue_for_next_pattern -= count;
                    query_position += count as usize;
                },
                AlignmentCase::Deletion => {
                    record_position += count as usize;
                },
            };
        });

        Self(record_position_to_next_patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_record_position_to_next_patterns() {
        let operations = vec![
            AlignmentOperation { case: AlignmentCase::Match, count: 50 },
            AlignmentOperation { case: AlignmentCase::Subst, count: 10 },
            AlignmentOperation { case: AlignmentCase::Match, count: 20 },
            AlignmentOperation { case: AlignmentCase::Insertion, count: 30 },
            AlignmentOperation { case: AlignmentCase::Match, count: 5 },
            AlignmentOperation { case: AlignmentCase::Deletion, count: 25 },
            AlignmentOperation { case: AlignmentCase::Match, count: 30 },
        ];

        let record_position_to_next_patterns = RecordPositionToNextPatterns::new(
            &operations, 0, 0, 20,
        );

        let answer = RecordPositionToNextPatterns(vec![
            Some(20), Some(40), None, Some(80), None, None, Some(135), None,
        ]);

        assert_eq!(record_position_to_next_patterns.0, answer.0);
    }
}


// ALGORITHM


// pub fn local_alignment_algorithm<S: SequenceProvider>(
//     reference: &Reference<S>,
//     sequence_buffer: &mut S::Buffer,
//     query: Sequence,
//     pattern_size: usize,
//     penalties: &Penalties,
//     cutoff: &Cutoff,
//     min_penalty_for_pattern: &MinPenaltyForPattern,
//     left_wave_front: &mut WaveFront,
//     right_wave_front: &mut WaveFront,
// ) -> AlignmentResult {
//     let anchors_preset_by_record = Anchors::create_preset_by_record(reference, query, pattern_size);

//     AlignmentResult(
//         anchors_preset_by_record.into_iter().filter_map(|(record_index, anchors_preset)| {
//             reference.fill_sequence_buffer(record_index, sequence_buffer);
//             let record_sequence = sequence_buffer.request_sequence();
//             let record_length = record_sequence.len();

//             let mut anchors = Anchors::from_preset(anchors_preset, record_length, query, pattern_size, cutoff, min_penalty_for_pattern);
//             anchors.extend(record_sequence, query, penalties, cutoff, left_wave_front, right_wave_front);
        
//             let alignment_results = anchors.get_alignment_results_for_local();

//             if alignment_results.len() == 0 {
//                 None
//             } else {
//                 Some(
//                     RecordAlignmentResult {
//                         index: record_index,
//                         alignments: alignment_results,
//                     }
//                 )
//             }
//         }).collect()
//     )
// }
