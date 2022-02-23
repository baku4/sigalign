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
    New(PenaltyMargin),
    Extending(ExtensionState),
    Evaluated(EvaluationState),
}

#[derive(Debug, Clone)]
enum ExtensionState {
    Res(Extension, TraversedAnchors), // Right Extension Success
    Ref(Extension, TraversedAnchors), // Right Extension Failed
    Rts(ExtensionReference), // Right Traversed Success
    Rtf(PenaltyMargin), // Right Traversed Failed
    Lts(ExtensionReference), // Left Traversed Success
    Ltf(PenaltyMargin), // Left Traversed Failed
    LesRts(Extension, TraversedAnchors, ExtensionReference), // Left Extension Success when Right Traversed Success
    LefRts(Extension, TraversedAnchors, ExtensionReference), // Left Extension Failed when Right Traversed Success
    LesRtf(Extension, TraversedAnchors), // Left Extension Success when Right Traversed Failed
    LefRtf(Extension, TraversedAnchors), // Left Extension Failed when Right Traversed Failed
    ResLts(Extension, TraversedAnchors, ExtensionReference), // Right Extension Success when Left Traversed Success
    RefLts(Extension, TraversedAnchors, ExtensionReference), // Right Extension Failed when Left Traversed Success
    ResLtf(Extension, TraversedAnchors), // Right Extension Success when Left Traversed Failed
    RefLtf(Extension, TraversedAnchors), // Right Extension Failed when Left Traversed Failed
}

#[derive(Debug, Clone)]
enum EvaluationState {
    Invalid,
    Ignored,
    Valid,
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

enum EvaluationResult {
    Done,
    RetainStack,
    PushStack(AnchorIndex),
}

impl StateTable {
    fn new(pos_table: &PosTable, min_penalty_for_pattern: &MinPenaltyForPattern, cutoff: &Cutoff, pattern_size: usize) -> Self {
        let pattern_count = pos_table.0.len();
        let left_penalty_margin_for_new_pattern: Vec<i64> = (0..pattern_count).map(|left_pattern_count| {
            let mut min_penalty = (left_pattern_count / 2) * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even);
            min_penalty += (left_pattern_count % 2) * min_penalty_for_pattern.odd;
            let then_max_length =  pattern_size * left_pattern_count + left_pattern_count;
            (then_max_length * cutoff.maximum_penalty_per_scale - min_penalty) as i64
        }).collect();

        Self(pos_table.0.iter().zip(left_penalty_margin_for_new_pattern.into_iter()).map(|(pattern_position, left_penalty_margin)| {
            vec![AnchorState::New(left_penalty_margin); pattern_position.len()]
        }).collect())
    }

    fn evaluate_anchor(
        &mut self,
        anchor_index: &AnchorIndex,
        pos_table: &PosTable,
        wave_front: &mut WaveFront,
    ) -> EvaluationResult {
        let current_state = &mut self.0[anchor_index.0][anchor_index.1];

        match current_state {
            AnchorState::New(penalty_margin) => {
                // let extension_result = pos_table.extend_wave_front_right(
                //     anchor_index,
                //     pattern_size,
                //     wave_front);
                // let traversed = self.check_traversed(&extension_result.extension);

                // if extension_result.is_success {
                //     current_anchor.state = AnchorState::Extending(
                //         ExtensionState::Res(
                //             extension_result.extension,
                //             traversed,
                //         )
                //     );
                // } else {
                //     current_anchor.state = AnchorState::Extending(
                //         ExtensionState::Ref(
                //             extension_result.extension,
                //             traversed,
                //         )
                //     );
                // }
                EvaluationResult::RetainStack
            },
            AnchorState::Extending(extension_state) => {
                match extension_state {
                    // ExtensionState::Res(extension, traversed) => {
                    //     match &traversed.at_the_end {
                    //         Some(other_anchor_index) => {
                    //             //TODO:
                    //             EvaluationResult::RetainStack
                    //         },
                    //         None => {
                    //             let extension_result = current_anchor.extend_left();
                    //             if extension_result.is_success {
                    //                 //TODO:
                    //                 EvaluationResult::RetainStack
                    //             } else {
                    //                 //TODO:
                    //                 EvaluationResult::RetainStack
                    //             }
                    //         },
                    //     }
                    // },
                    _ => {
                        //TODO:
                        EvaluationResult::RetainStack
                    },
                }
            },
            AnchorState::Evaluated(_) => {
                EvaluationResult::Done
            },
        }
    }
}

impl PosTable {
    fn extend_right(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        penalty_margin: i64,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        //
    }
}
