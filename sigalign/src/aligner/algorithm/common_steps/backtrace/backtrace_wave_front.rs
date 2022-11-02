use super::{
	Penalties,
    AlignmentOperation, AlignmentCase,
};

use super::{Extension, WaveFront, BackTraceMarker};
use super::{TraversedPosition};

impl WaveFront {
    pub fn backtrace_from_point_checking_right_traversed(
        &self,
        mut score: usize,
        index_of_component: usize,
        penalties: &Penalties,
        pattern_size: usize,
    ) -> (Extension, Vec<TraversedPosition>) {
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
        let mut traversed_positions = Vec::new();

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
                            let pattern_count_to_next_pattern = query_index_of_subst / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: pattern_count_to_next_pattern as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score + penalties.x,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            let pattern_count_to_next_pattern = query_index_of_ins / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: pattern_count_to_next_pattern as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            let pattern_count_to_next_pattern = query_index_of_del / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: pattern_count_to_next_pattern as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            return (extension, traversed_positions);
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
    pub fn backtrace_from_point_checking_left_traversed(
        &self,
        mut score: usize,
        index_of_component: usize,
        penalties: &Penalties,
        pattern_size: usize,
    ) -> (Extension, Vec<TraversedPosition>) {
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
        let mut traversed_positions = Vec::new();

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
                            let pattern_count_to_next_pattern = query_index_of_subst / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: (pattern_count_to_next_pattern + traversed_pattern_count) as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k + anchor_size) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score + penalties.x,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            let pattern_count_to_next_pattern = query_index_of_ins / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: (pattern_count_to_next_pattern + traversed_pattern_count) as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k + anchor_size) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            let pattern_count_to_next_pattern = query_index_of_del / pattern_size as i32 + 1;
                            let query_slice_index_of_next_pattern = pattern_count_to_next_pattern * pattern_size as i32;
                            let alternative_match_count = fr - k - query_slice_index_of_next_pattern; // fr-k: query slice index of next unmatched
                            let traversed_pattern_count = alternative_match_count / pattern_size as i32;

                            if traversed_pattern_count > 0 {
                                let anchor_size = traversed_pattern_count * pattern_size as i32;

                                let traversed_position = TraversedPosition {
                                    pattern_count_from_start_point: (pattern_count_to_next_pattern + traversed_pattern_count) as usize,
                                    traversed_record_length_to_anchor: (query_slice_index_of_next_pattern + k + anchor_size) as usize,
                                    traversed_length_to_anchor_end: (query_slice_index_of_next_pattern + k + anchor_size) as usize + component.deletion_count as usize,
                                    traversed_penalty_to_anchor_end: score,
                                    index_of_operation: operations.len(),
                                    alternative_match_count: (alternative_match_count - anchor_size) as u32,
                                };
                                traversed_positions.push(traversed_position);
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
                            return (extension, traversed_positions);
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