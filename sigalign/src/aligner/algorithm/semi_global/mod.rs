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

struct StateTable(Vec<Vec<AnchorState>>);

#[derive(Debug, Clone)]
enum AnchorState {
    
}

#[derive(Debug, Clone)]
struct ExtensionReference {
    penalty: usize,
    length: usize,
    insertion_count: u32,
    deletion_count: u32,
    anchor_index: AnchorIndex,
    index_of_operation: usize,
    alternative_match_count: u32,
}

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
