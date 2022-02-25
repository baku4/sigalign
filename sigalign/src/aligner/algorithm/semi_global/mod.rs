use std::borrow::Borrow;

use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

type PenaltyMargin = i64;

mod extending;
use extending::ExtensionResult;

struct AnchorTable(Vec<Vec<Anchor>>);

#[derive(Debug, Clone)]
struct Anchor {
    left_extension: Option<ExtensionState>,
    right_extension: Option<ExtensionState>,
    leftmost_anchor_index: Option<AnchorIndex>,
    rightmost_anchor_index: Option<AnchorIndex>,
    is_invalid: bool,
}

#[derive(Debug, Clone)]
enum ExtensionState {
    ExtensionSuccess(Extension),
    TraversedSuccess(TraversedAnchor),
    TraversedFail(usize), // min penalty
}

#[derive(Debug, Clone)]
struct ExtensionReference {
    anchor_from: AnchorIndex,
    penalty: usize,
    length: usize,
    index_of_operation: usize,
    alternative_match_count: u32,
}

impl AnchorTable {
    fn new(pos_table: &PosTable, min_penalty_for_pattern: &MinPenaltyForPattern, cutoff: &Cutoff, pattern_size: usize) {

    }
    fn extend(
        &mut self,
        pos_table: &PosTable,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        let pattern_count = pos_table.0.len();
        let left_penalty_margin_for_new_pattern: Vec<i64> = (0..pattern_count).map(|left_pattern_count| {
            let mut min_penalty = (left_pattern_count / 2) * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even);
            min_penalty += (left_pattern_count % 2) * min_penalty_for_pattern.odd;
            let then_max_length =  pattern_size * left_pattern_count + left_pattern_count;
            (then_max_length * cutoff.maximum_penalty_per_scale - min_penalty) as i64
        }).collect();

        let anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
            (0..pattern_position.len()).map(move |anchor_index| {
                (pattern_index, anchor_index)
            })
        }).flatten().collect();

        // Right Extension
        anchor_indices.iter().for_each(|current_anchor_index| {
            let need_right_extension = self.0[current_anchor_index.0][current_anchor_index.1].right_extension.is_none();
            if need_right_extension {
                let left_penalty_margin = left_penalty_margin_for_new_pattern[current_anchor_index.0];

                let right_extension_result = pos_table.right_extension(
                    current_anchor_index,
                    pattern_size,
                    left_penalty_margin,
                    record_sequence,
                    query_sequence,
                    penalties,
                    cutoff,
                    wave_front,
                );

                if right_extension_result.is_success {
                    let rightmost_anchor_index = match right_extension_result.traversed_anchors.last() {
                        Some(v) => Some(v.anchor_index),
                        None => None,
                    };
                    // Traversed anchors
                    for extension_result_of_traversed_anchor in &right_extension_result.traversed_anchors {
                        let traversed_anchor_index = extension_result_of_traversed_anchor.anchor_index;
                        let traversed_anchor = &mut self.0[traversed_anchor_index.0][traversed_anchor_index.1];
                        traversed_anchor.right_extension = Some(ExtensionState::TraversedSuccess(extension_result_of_traversed_anchor.clone()));
                        traversed_anchor.rightmost_anchor_index = rightmost_anchor_index;
                    }
                    // Current anchor
                    let current_anchor = &mut self.0[current_anchor_index.0][current_anchor_index.1];
                    current_anchor.right_extension = Some(ExtensionState::ExtensionSuccess(right_extension_result.extension));
                    current_anchor.rightmost_anchor_index = rightmost_anchor_index;
                } else {
                    // Current anchor
                    self.0[current_anchor_index.0][current_anchor_index.1].is_invalid = true;
                    // Traversed anchors
                    for traversed_anchor in right_extension_result.traversed_anchors {
                        let traversed_anchor_index = traversed_anchor.anchor_index;
                        let traversed_anchor_right_extension = ExtensionState::TraversedFail(traversed_anchor.remained_penalty);
                        self.0[traversed_anchor_index.0][traversed_anchor_index.1].right_extension = Some(traversed_anchor_right_extension);
                    }
                }
            }            
        });

        // Left Extension
        anchor_indices.iter().rev().for_each(|current_anchor_index| {
            let need_left_extension = {
                let current_anchor = &self.0[current_anchor_index.0][current_anchor_index.1];
                !current_anchor.is_invalid && current_anchor.left_extension.is_none()
            };
            
            if need_left_extension {
                let right_penalty_margin = {
                    let current_anchor = &self.0[current_anchor_index.0][current_anchor_index.1];
                    match current_anchor.right_extension.as_ref().unwrap() {
                        ExtensionState::ExtensionSuccess(extension) => {
                            (extension.length * cutoff.maximum_penalty_per_scale) as i64 - extension.penalty as i64
                        },
                        ExtensionState::TraversedSuccess(traversed_anchor) => {
                            (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - traversed_anchor.remained_penalty as i64
                        },
                        ExtensionState::TraversedFail(minimum_penalty) => {
                            let right_anchor_position = &pos_table.0[current_anchor_index.0][current_anchor_index.1];
                            let pattern_count = right_anchor_position.pattern_count;

                            let query_start_index = (current_anchor_index.0 + pattern_count) * pattern_size;
                            let record_start_index = right_anchor_position.record_position + pattern_count * pattern_size;

                            let query_length = query_sequence.len() - query_start_index;
                            let record_length = record_sequence.len() - record_start_index;

                            let min_length = query_length.min(record_length);
                            let max_gap = (min_length - penalties.o) / penalties.x;

                            ((min_length + max_gap) * cutoff.maximum_penalty_per_scale) as i64 - *minimum_penalty as i64
                        },
                    }
                };
                
                let left_extension_result = pos_table.left_extension(
                    current_anchor_index,
                    pattern_size,
                    right_penalty_margin,
                    record_sequence,
                    query_sequence,
                    penalties,
                    cutoff,
                    wave_front,
                );

                if left_extension_result.is_success {
                    match left_extension_result.traversed_anchors.last() {
                        Some(v) => Some(v.anchor_index),
                        None => None,
                    };
                    //TODO: Next
                } else {
                    //TODO: Next
                }
            }            
        });
    }
}

impl ExtensionReference {
    fn new(original_anchor_index: &AnchorIndex, traversed_anchor: &TraversedAnchor) -> Self {
        Self {
            anchor_from: original_anchor_index.clone(),
            penalty: traversed_anchor.remained_penalty,
            length: traversed_anchor.remained_length,
            index_of_operation: traversed_anchor.index_of_operation,
            alternative_match_count: traversed_anchor.alternative_match_count,
        }
    }
}


#[derive(Debug, Clone)]
enum AnchorState {
    
}

struct StateTable(Vec<Vec<AnchorState>>);


// impl StateTable {
//     fn new(pos_table: &PosTable, min_penalty_for_pattern: &MinPenaltyForPattern, cutoff: &Cutoff, pattern_size: usize) -> Self {
//         let pattern_count = pos_table.0.len();
//         let left_penalty_margin_for_new_pattern: Vec<i64> = (0..pattern_count).map(|left_pattern_count| {
//             let mut min_penalty = (left_pattern_count / 2) * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even);
//             min_penalty += (left_pattern_count % 2) * min_penalty_for_pattern.odd;
//             let then_max_length =  pattern_size * left_pattern_count + left_pattern_count;
//             (then_max_length * cutoff.maximum_penalty_per_scale - min_penalty) as i64
//         }).collect();

//         Self(pos_table.0.iter().zip(left_penalty_margin_for_new_pattern.into_iter()).map(|(pattern_position, left_penalty_margin)| {
//             vec![AnchorState::New(left_penalty_margin); pattern_position.len()]
//         }).collect())
//     }

//     fn evaluate_anchor(
//         &mut self,
//         anchor_index: &AnchorIndex,
//         pos_table: &PosTable,
//         pattern_size: usize,
//         record_sequence: Sequence,
//         query_sequence: Sequence,
//         penalties: &Penalties,
//         cutoff: &Cutoff,
//         wave_front: &mut WaveFront,
//     ) -> EvaluationResult {
//         let (left_states, right_states) = self.0.split_at_mut(anchor_index.0);
//         let (mid_states, right_states) = right_states.split_first_mut().unwrap();

//         let current_state = &mut mid_states[anchor_index.1];

//         match current_state {
//             AnchorStateDep::New(penalty_margin) => {
//                 let extension_result = pos_table.right_extension(
//                     anchor_index,
//                     pattern_size,
//                     *penalty_margin,
//                     record_sequence,
//                     query_sequence,
//                     penalties,
//                     cutoff,
//                     wave_front,
//                 );

//                 if extension_result.is_success {
//                     match extension_result.traversed_anchors.last() {
//                         Some(rightmost_traversed_anchor) => {
//                             // Current state
//                             *current_state = AnchorStateDep::Extending(
//                                 ExtensionState::Res(
//                                     extension_result.extension,
//                                     extension_result.traversed_anchors,
//                                     Some(rightmost_traversed_anchor.anchor_index),
//                                 )
//                             );
//                             // Evaluation Result
//                             EvaluationResult::Push(rightmost_traversed_anchor.anchor_index)
//                         },
//                         None => {
//                             // Current state
//                             *current_state = AnchorStateDep::Extending(
//                                 ExtensionState::Res(
//                                     extension_result.extension,
//                                     extension_result.traversed_anchors,
//                                     None,
//                                 )
//                             );
//                             // Evaluation Result
//                             EvaluationResult::Retain
//                         },
//                     }
//                 } else {
//                     match extension_result.traversed_anchors.last() {
//                         Some(rightmost_traversed_anchor) => {
//                             // Current state
//                             *current_state = AnchorStateDep::Extending(
//                                 ExtensionState::Ref(
//                                     traversed,
//                                 )
//                             );
//                             EvaluationResult::Retain
//                         },
//                         None => {
//                             rightmost_traversed_anchor
//                         },
//                     }
//                 }
//             },
//             AnchorStateDep::Extending(extension_state) => {
//                 match extension_state {
//                     // ExtensionState::Res(extension, traversed) => {
//                     //     match &traversed.at_the_end {
//                     //         Some(other_anchor_index) => {
//                     //             //TODO:
//                     //             EvaluationResult::RetainStack
//                     //         },
//                     //         None => {
//                     //             let extension_result = current_anchor.extend_left();
//                     //             if extension_result.is_success {
//                     //                 //TODO:
//                     //                 EvaluationResult::RetainStack
//                     //             } else {
//                     //                 //TODO:
//                     //                 EvaluationResult::RetainStack
//                     //             }
//                     //         },
//                     //     }
//                     // },
//                     _ => {
//                         //TODO:
//                         EvaluationResult::Retain
//                     },
//                 }
//             },
//             AnchorStateDep::Evaluated(_) => {
//                 EvaluationResult::Done
//             },
//         }
//     }
// }

#[cfg(test)]
mod tests {
    #[test]
    fn test_split() {
        let mut vec = vec![0,1,2,3,4,5,6,7,8,9];

        for i in 0..10 {
            let (left, right) = vec.split_at_mut(i);
            let (mid, right) = right.split_first_mut().unwrap();

            println!("left: {:?}", left);
            println!("mid: {:?}", mid);
            println!("right: {:?}", right);
        }
       
    }
}
