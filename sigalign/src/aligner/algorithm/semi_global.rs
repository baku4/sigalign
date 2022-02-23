use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedAnchors};

use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

struct AnchorTable {
    pos_table: PosTable,
    state_table: StateTable,
}

struct StateTable(Vec<Vec<AnchorState>>);

#[derive(Debug, Clone)]
enum AnchorState {
    New,
    Extending(ExtensionState),
    Evaluated(EvaluationState),
}

#[derive(Debug, Clone)]
enum ExtensionState {
    Res(Extension, TraversedAnchors), // Right Extension Success
    Ref(Extension, TraversedAnchors), // Right Extension Failed
    Rts(ExtensionReference), // Right Traversed Success
    Rtf, // Right Traversed Failed
    Lts(ExtensionReference), // Left Traversed Success
    Ltf, // Left Traversed Failed
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
    start_point: OperationStartPoint,
}

#[derive(Debug, Clone)]
struct OperationStartPoint {
    index: usize,
    count: u32,
}

struct ExtensionResult {
    extension: Extension,
    is_success: bool,
}

enum EvaluationResult {
    Done,
    RetainStack,
    PushStack(AnchorIndex),
}

impl StateTable {
    fn new(pos_table: &PosTable) -> Self {
        Self(pos_table.0.iter().map(|pattern_position| {
            vec![AnchorState::New; pattern_position.len()]
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
            AnchorState::New => {
                // let extension_result = pos_table.extend_right(anchor_index, wave_front);
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
