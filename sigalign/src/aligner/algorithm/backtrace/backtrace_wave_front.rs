use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

pub type TraversedPositions = Vec<Option<usize>>;

impl WaveFront {
    pub fn backtrace_from_the_end(&self, penalties: &Penalties) -> Option<Extension> {
        match self.end_point.k {
            Some(k) => {
                let last_score = self.end_point.score;
                let index_of_component = (self.wave_front_scores[last_score].max_k + k) as usize;
                Some(self.backtrace_from_point(last_score, index_of_component, penalties))
            },
            None => {
                None
            },
        }
    }

    pub fn backtrace_from_point(
        &self,
        mut score: usize,
        index_of_component: usize,
        penalties: &Penalties,
    ) -> Extension {
        let penalty_from_start_point = score;

        let wave_front_scores = &self.wave_front_scores;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        
        let mut wave_front_score = &wave_front_scores[score];

        // Init
        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[index_of_component].m;

        let mut k = -wave_front_score.max_k + index_of_component as i32;
        let mut fr = component.fr;

        let operation_length = fr as usize + component.deletion_count as usize;
        let deletion_count: u32 = component.deletion_count as u32;
        let insertion_count: u32 = (deletion_count as i32 + k) as u32;
        
        loop {
            match component_type {
                /* M */
                ComponentType::M => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            let match_count = (fr - next_fr - 1) as u32;
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperation {
                                        case: AlignmentCase::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperation {
                                            case: AlignmentCase::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Subst,
                                        count: 1
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromI => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::I;
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromD => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::D;
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add Cigar
                            let match_count = (fr-next_fr) as u32;
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            if fr != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: fr as u32,
                                    }
                                );
                            };
                            // shrink
                            operations.shrink_to_fit();
                            // extension of current anchor
                            let extension = Extension {
                                penalty: penalty_from_start_point,
                                length: operation_length,
                                insertion_count,
                                deletion_count,
                                operations: operations,
                            };
                            return extension;
                        }
                    }
                },
                /* I */
                ComponentType::I => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
                /* D */
                ComponentType::D => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
    pub fn backtrace_from_point_checking_traversed(
        &self,
        mut score: usize,
        index_of_component: usize,
        penalties: &Penalties,
        pattern_size: usize,
    ) -> (Extension, TraversedPositions) {
        let penalty_from_start_point = score;

        let wave_front_scores = &self.wave_front_scores;
        let mut operations: Vec<AlignmentOperation> = Vec::new(); // TODO: Capacity can be applied?
        
        let mut wave_front_score = &wave_front_scores[score];

        // Init
        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[index_of_component].m;

        let mut k = -wave_front_score.max_k + index_of_component as i32;
        let mut fr = component.fr;

        let operation_length = fr as usize + component.deletion_count as usize;
        let deletion_count: u32 = component.deletion_count as u32;
        let insertion_count: u32 = (deletion_count as i32 + k) as u32;

        // For checking traversed
        let query_length = (fr - k) as usize;
        let pattern_count = (query_length - 1) / pattern_size;
        let mut record_position_by_pattern: TraversedPositions = vec![None; pattern_count + 1];
        
        loop {
            match component_type {
                /* M */
                ComponentType::M => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Check traversed
                            let match_count = (fr - next_fr - 1) as u32;
                            let query_index_of_subst = next_fr - k;
                            let pattern_index_of_next_pattern = query_index_of_subst / pattern_size as i32 + 1;
                            let query_index_of_next_pattern = pattern_index_of_next_pattern * pattern_size as i32;
                            if query_index_of_next_pattern + pattern_size as i32 <= fr - k {
                                record_position_by_pattern[pattern_index_of_next_pattern as usize] = Some((query_index_of_next_pattern + k) as usize);
                            }
                            // (8) Add operation
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperation {
                                        case: AlignmentCase::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperation {
                                            case: AlignmentCase::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Subst,
                                        count: 1
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromI => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::I;
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Check traversed
                            let match_count = (fr-next_fr) as u32;
                            let query_index_of_ins = next_fr - k - 1;
                            let pattern_index_of_next_pattern = query_index_of_ins / pattern_size as i32 + 1;
                            let query_index_of_next_pattern = pattern_index_of_next_pattern * pattern_size as i32;
                            if query_index_of_next_pattern + pattern_size as i32 <= fr - k {
                                record_position_by_pattern[pattern_index_of_next_pattern as usize] = Some((query_index_of_next_pattern + k) as usize);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromD => {
                            // (1) Next score
                            // not change
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            // not change
                            // (4) Component type
                            component_type = ComponentType::D;
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Check traversed
                            let match_count = (fr-next_fr) as u32;
                            let query_index_of_del = next_fr - k - 1;
                            let pattern_index_of_next_pattern = query_index_of_del / pattern_size as i32 + 1;
                            let query_index_of_next_pattern = pattern_index_of_next_pattern * pattern_size as i32;
                            if query_index_of_next_pattern + pattern_size as i32 <= fr - k {
                                record_position_by_pattern[pattern_index_of_next_pattern as usize] = Some((query_index_of_next_pattern + k) as usize);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: match_count
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            // Add operation
                            if fr != 0 {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Match,
                                        count: fr as u32,
                                    }
                                );
                            };
                            // shrink
                            operations.shrink_to_fit(); // TODO: Is needed?
                            // extension of current anchor
                            let extension = Extension {
                                penalty: penalty_from_start_point,
                                length: operation_length,
                                insertion_count,
                                deletion_count,
                                operations: operations,
                            };
                            return (extension, record_position_by_pattern);
                        }
                    }
                },
                /* I */
                ComponentType::I => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
                /* D */
                ComponentType::D => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next score
                            score -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next score
                            score -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[score];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperation {
                                    case: AlignmentCase::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperation {
                                        case: AlignmentCase::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
}

enum ComponentType {
    M,
    I,
    D,
}