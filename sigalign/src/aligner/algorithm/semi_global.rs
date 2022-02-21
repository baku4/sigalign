use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, PatternPosition, AnchorPosition};

use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty_from_determinant};

struct AnchorTable(Vec<Vec<Anchor>>);

type AnchorIndex = (usize, usize);

struct Anchor {
    record_position: usize,
    pattern_count: usize,
    anchor_size: usize,
    state: AnchorState,
}

enum AnchorState {
    New,
    Extending(ExtensionState),
    Evaluated(EvaluationState),
}

enum ExtensionState {
    Res(Extension, Traversed), // Right Extension Success
    Ref(Extension, Traversed), // Right Extension Failed
    Rts(ExtensionReference), // Right Traversed Success
    Rtf, // Right Traversed Failed
    Lts(ExtensionReference), // Left Traversed Success
    Ltf, // Left Traversed Failed
    LesRts(Extension, Traversed, ExtensionReference), // Left Extension Success when Right Traversed Success
    LefRts(Extension, Traversed, ExtensionReference), // Left Extension Failed when Right Traversed Success
    LesRtf(Extension, Traversed), // Left Extension Success when Right Traversed Failed
    LefRtf(Extension, Traversed), // Left Extension Failed when Right Traversed Failed
    ResLts(Extension, Traversed, ExtensionReference), // Right Extension Success when Left Traversed Success
    RefLts(Extension, Traversed, ExtensionReference), // Right Extension Failed when Left Traversed Success
    ResLtf(Extension, Traversed), // Right Extension Success when Left Traversed Failed
    RefLtf(Extension, Traversed), // Right Extension Failed when Left Traversed Failed
}

enum EvaluationState {
    Invalid,
    Ignored,
    Valid,
}

#[derive(Debug, Clone)]
struct Traversed {
    sorted_list: Vec<AnchorIndex>,
    at_the_end: Option<AnchorIndex>,
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

impl AnchorTable {
    fn new(pos_table: PosTable, pattern_size: usize) -> Self {
        Self(pos_table.position_by_pattern.into_iter().map(|optional_pattern_position| {
            match optional_pattern_position {
                Some(pattern_position) => {
                    pattern_position.anchor_positions.into_iter().map(|anchor_position| {
                        Anchor::new(anchor_position.record_position, anchor_position.pattern_count, pattern_size)
                    }).collect()
                },
                None => {
                    Vec::new()
                },
            }
        }).collect())
    }
    fn check_traversed(&self, extension: &Extension) -> Traversed {
        //TODO:
        Traversed {
            sorted_list: Vec::new(),
            at_the_end: None,
        }
    }

    fn evaluate_anchor(
        &mut self,
        anchor_index: &AnchorIndex,
    ) -> EvaluationResult {
        let current_anchor = &mut self.0[anchor_index.0][anchor_index.1];

        match &mut current_anchor.state {
            AnchorState::New => {
                let extension_result = current_anchor.extend_right();
                let traversed = self.check_traversed(&extension_result.extension);

                if extension_result.is_success {
                    current_anchor.state = AnchorState::Extending(
                        ExtensionState::Res(
                            extension_result.extension,
                            traversed,
                        )
                    );
                } else {
                    current_anchor.state = AnchorState::Extending(
                        ExtensionState::Ref(
                            extension_result.extension,
                            traversed,
                        )
                    );
                }
                EvaluationResult::RetainStack
            },
            AnchorState::Extending(extension_state) => {
                match extension_state {
                    ExtensionState::Res(extension, traversed) => {
                        match &traversed.at_the_end {
                            Some(other_anchor_index) => {
                                //TODO:
                                EvaluationResult::RetainStack
                            },
                            None => {
                                let extension_result = current_anchor.extend_left();
                                if extension_result.is_success {
                                    //TODO:
                                    EvaluationResult::RetainStack
                                } else {
                                    //TODO:
                                    EvaluationResult::RetainStack
                                }
                            },
                        }
                    },
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
    
    fn extend_and_evaluate(&mut self) {

    }
}



impl Anchor {
    fn new(record_position: usize, pattern_count: usize, pattern_size: usize) -> Self {
        //TODO:
        Self {
            record_position: 0,
            pattern_count: 0,
            anchor_size: 0,
            state: AnchorState::New,
        }
    }

    fn extend_right(&mut self) -> ExtensionResult {
        //TODO:
        ExtensionResult::dummy()
    }

    fn extend_left(&mut self) -> ExtensionResult {
        //TODO:
        ExtensionResult::dummy()
    }
}

impl Traversed {

}

impl ExtensionResult {
    fn dummy() -> Self {
        Self {
            extension: Extension {
                penalty: 0,
                length: 0,
                insertion_count: 0,
                deletion_count: 0,
                operations: vec![
                    AlignmentOperation {
                        case: AlignmentCase::Match,
                        count: 0,
                    }
                ]
            },
            is_success: true,
        }
    }
}
