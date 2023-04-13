use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
    }
};
use super::{AnchorTable, Anchor, AnchorIndex};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker};
use super::SideExtension;
use super::Vpc;
use ahash::AHashSet;
use num::integer::div_rem;

#[derive(Debug, Clone)]
pub struct TraversedPosition {
    pub operation_length_from_the_start: u32,
    pub penalty_from_the_start: u32,
    pub estimated_additive_pattern_index: u32,
    pub estimated_additive_target_position: u32,
    pub partial_operation_start_index: u32,
    pub alternative_match_count: u32,
}

#[derive(Debug, Clone)]
pub struct TraversedPositionDep {
    pub scaled_penalty_delta_from_the_end: i64,
    pub penalty_from_the_start: u32,
    pub estimated_additive_pattern_index: u32,
    pub estimated_additive_target_position: u32,
    pub partial_operation_index: u32,
    pub alternative_match_count: u32,
}

enum ComponentType {
    M,
    I,
    D,
}

// TODO: Backtrace can refer the other extensions of this anchor
impl WaveFront {
    #[inline]
    pub fn backtrace_of_left_side(
        &self,
        mut penalty: u32,
        pattern_size: u32,
        component_index: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
        traversed_positions_buffer: &mut Vec<TraversedPositionDep>,
    ) -> SideExtension {
        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let mut operations: Vec<AlignmentOperations> = Vec::new(); // TODO: Capacity can be applied?

        let total_penalty = penalty;
        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

        let total_operation_length = fr as u32 + component.deletion_count as u32;
        let total_deletion_count: u32 = component.deletion_count as u32;
        let total_insertion_count: u32 = (total_deletion_count as i32 + k) as u32;

        loop {
            match component_type {
                /* M */
                ComponentType::M => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next penalty
                            penalty -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Check traversed
                            let match_count = fr - next_fr - 1;

                            let (quotient, remainder) = div_rem(fr-k, pattern_size as i32); // fr - k = query_index_of_first_match
                            let match_count_of_next_pattern = match_count - remainder;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let penalty_from_the_start = penalty + penalties.x;
                                let operation_length = total_operation_length - match_count_of_next_pattern as u32 - 1 - next_fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64 
                                    - PREC_SCALE as i64 * (total_penalty - penalty_from_the_start) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start,
                                    estimated_additive_pattern_index: quotient as u32,
                                    estimated_additive_target_position: (next_fr + match_count_of_next_pattern + 1) as u32,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: (match_count - match_count_of_next_pattern) as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            
                            // (8) Add operation
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperations {
                                            operation: AlignmentOperation::Subst,
                                            count: 1,
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
                                    }
                                );
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: 1,
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromI => {
                            // (1) Next penalty
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
                            let match_count = fr - next_fr;

                            let (quotient, remainder) = div_rem(fr - k, pattern_size as i32);
                            let match_count_of_next_pattern = match_count - remainder;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let operation_length = total_operation_length - match_count_of_next_pattern as u32 - next_fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64 
                                    - PREC_SCALE as i64 * (total_penalty - penalty) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start: penalty,
                                    estimated_additive_pattern_index: quotient as u32,
                                    estimated_additive_target_position: (next_fr + match_count_of_next_pattern) as u32,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: (match_count - match_count_of_next_pattern) as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromD => {
                            // (1) Next penalty
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
                            let match_count = fr-next_fr;

                            let (quotient, remainder) = div_rem(fr - k, pattern_size as i32);
                            let match_count_of_next_pattern = match_count - remainder;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let operation_length = total_operation_length - match_count_of_next_pattern as u32 - next_fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64 
                                    - PREC_SCALE as i64 * (total_penalty - penalty) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start: penalty,
                                    estimated_additive_pattern_index: quotient as u32,
                                    estimated_additive_target_position: (next_fr + match_count_of_next_pattern) as u32,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: (match_count - match_count_of_next_pattern) as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
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
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: fr as u32,
                                    }
                                );
                            };

                            // extension of current anchor
                            let extension = SideExtension {
                                penalty: total_penalty,
                                length: total_operation_length,
                                insertion_count: total_insertion_count,
                                deletion_count: total_deletion_count,
                                reversed_operations: operations,
                                traversed_anchors: Vec::new(),
                                query_index_of_the_end: 0,
                            };
                            return extension;
                        }
                    }
                },
                /* I */
                ComponentType::I => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next penalty
                            penalty -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next penalty
                            penalty -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
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
                            // (1) Next penalty
                            penalty -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next penalty
                            penalty -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
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
    #[inline]
    pub fn backtrace_of_right_side(
        &self,
        mut penalty: u32,
        pattern_size: u32,
        pattern_count_of_anchor: u32,
        component_index: u32,
        maximum_scaled_penalty_per_length: u32,
        penalties: &Penalty,
        traversed_positions_buffer: &mut Vec<TraversedPositionDep>,
    ) -> SideExtension {
        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let anchor_size = pattern_count_of_anchor * pattern_size;
        let mut operations: Vec<AlignmentOperations> = Vec::new(); // TODO: Capacity can be applied?

        let total_penalty = penalty;
        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

        let total_operation_length = fr as u32 + component.deletion_count as u32;
        let total_deletion_count: u32 = component.deletion_count as u32;
        let total_insertion_count: u32 = (total_deletion_count as i32 + k) as u32;

        loop {
            match component_type {
                /* M */
                ComponentType::M => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next penalty
                            penalty -= penalties.x;
                            // (2) Next k
                            // not change
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Check traversed
                            let prev_match_query_index = next_fr - k;
                            let match_count = fr - next_fr - 1;
                            
                            let (quotient, remainder) = div_rem(prev_match_query_index, pattern_size as i32);
                            let match_count_of_next_pattern = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let penalty_from_the_start = penalty + penalties.x;
                                let operation_length = total_operation_length + match_count_of_next_pattern as u32 - fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64
                                    - PREC_SCALE as i64 * (total_penalty - penalty_from_the_start) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start,
                                    estimated_additive_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    estimated_additive_target_position: (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: match_count_of_next_pattern as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            
                            // (8) Add operation
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: last_fr
                                    }) = operations.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations.push(
                                        AlignmentOperations {
                                            operation: AlignmentOperation::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32
                                    }
                                );
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: 1
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromI => {
                            // (1) Next penalty
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
                            let match_count = fr - next_fr;
                            let prev_match_query_index = next_fr - k - 1;

                            let (quotient, remainder) = div_rem(prev_match_query_index, pattern_size as i32);
                            let match_count_of_next_pattern = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let operation_length = total_operation_length + match_count_of_next_pattern as u32 - fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64 
                                    - PREC_SCALE as i64 * (total_penalty - penalty) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start: penalty,
                                    estimated_additive_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    estimated_additive_target_position: (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: match_count_of_next_pattern as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        BackTraceMarker::FromD => {
                            // (1) Next penalty
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
                            let match_count = fr - next_fr;
                            let prev_match_query_index = next_fr - k - 1;

                            let (quotient, remainder) = div_rem(prev_match_query_index, pattern_size as i32);
                            let match_count_of_next_pattern = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                let operation_length = total_operation_length + match_count_of_next_pattern as u32 - fr as u32 - component.deletion_count as u32;
                                let scaled_penalty_delta_from_the_end = 
                                    operation_length as i64 * maximum_scaled_penalty_per_length as i64 
                                    - PREC_SCALE as i64 * (total_penalty - penalty) as i64
                                ;
                                let traversed_position = TraversedPositionDep {
                                    scaled_penalty_delta_from_the_end,
                                    penalty_from_the_start: penalty,
                                    estimated_additive_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    estimated_additive_target_position: (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                    partial_operation_index: operations.len() as u32,
                                    alternative_match_count: match_count_of_next_pattern as u32,
                                };
                                traversed_positions_buffer.push(traversed_position);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
                                    }
                                );
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            // Add operation
                            let last_match_count = fr as u32 + anchor_size;
                            operations.push(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Match,
                                    count: last_match_count,
                                }
                            );
                            // extension of current anchor
                            let extension = SideExtension {
                                penalty: total_penalty,
                                length: total_operation_length + anchor_size,
                                insertion_count: total_insertion_count,
                                deletion_count: total_deletion_count,
                                reversed_operations: operations,
                                traversed_anchors: Vec::new(),
                                query_index_of_the_end: 0,
                            };
                            return extension;
                        }
                    }
                },
                /* I */
                ComponentType::I => {
                    match component.bt {
                        BackTraceMarker::FromM => {
                            // (1) Next penalty
                            penalty -= penalties.o + penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_I
                            // (1) Next penalty
                            penalty -= penalties.e;
                            // (2) Next k
                            k -= 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
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
                            // (1) Next penalty
                            penalty -= penalties.o + penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            component_type = ComponentType::M;
                            // (5) Next component
                            component = wave_front_score.m_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
                                        count: 1,
                                    }
                                )
                            }
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // FROM_D
                            // (1) Next penalty
                            penalty -= penalties.e;
                            // (2) Next k
                            k += 1;
                            // (3) Next WFS
                            wave_front_score = &wave_front_scores[penalty as usize];
                            // (4) Component type
                            // not change
                            // (5) Next component
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
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





// FIXME: To dep

/*
pub struct VpcIndexPackageDep {
    pub left_vpc_indices: Vec<usize>,
    pub right_vpc_indices: Vec<usize>,
}

impl Vpc {
    // The result is sorted by left to right
    #[inline]
    pub fn package_vpc_index(
        left_sorted_vpc_vector: &Vec<Self>,
        right_sorted_vpc_vector: &Vec<Self>,
        anchor_scaled_penalty_delta: i64,
    ) -> Vec<VpcIndexPackageDep> {
        let mut left_vpc_checkpoints = Vec::new();
        let mut right_vpc_checkpoints = Vec::new();

        let mut right_vpc_index = 0;
        let mut left_vpc_index = left_sorted_vpc_vector.len(); // This time: last index + 1

        'outer: while (left_vpc_index > 0) && (right_vpc_index < right_sorted_vpc_vector.len()) {
            left_vpc_index -= 1; // Adjust
            // (1) Get left vpc index of checkpoint
            let rpd_with_anchor = right_sorted_vpc_vector[right_vpc_index].scaled_penalty_delta + anchor_scaled_penalty_delta;
            let mut pd = left_sorted_vpc_vector[left_vpc_index].scaled_penalty_delta + rpd_with_anchor;
            while pd < 0 {
                if left_vpc_index == 0 {
                    break 'outer;
                }
                left_vpc_index -= 1;
                pd = left_sorted_vpc_vector[left_vpc_index].scaled_penalty_delta + rpd_with_anchor;
            }
            // (2) Get right vpc index of checkpoint
            let lpd_with_anchor = left_sorted_vpc_vector[left_vpc_index].scaled_penalty_delta + anchor_scaled_penalty_delta;
            // In the below section:
            //  PD is next PD when next 'right_vpc' is exists.
            while pd >= 0 {
                right_vpc_index += 1;
                if right_vpc_index == right_sorted_vpc_vector.len() {
                    break;
                }
                pd = lpd_with_anchor + right_sorted_vpc_vector[right_vpc_index].scaled_penalty_delta;
            }
            // (3) Add index to checkpoints
            left_vpc_checkpoints.push(left_vpc_index);
            right_vpc_checkpoints.push(right_vpc_index-1);
        }

        let checkpoint_count = left_vpc_checkpoints.len();
        let mut start_vpc_index = 0;
        let mut left_packages = Vec::with_capacity(checkpoint_count);
        for last_vpc_index in left_vpc_checkpoints.into_iter().rev() {
            left_packages.push((start_vpc_index..=last_vpc_index).collect::<Vec<usize>>());
            start_vpc_index += 1;
        }
        let mut start_vpc_index = 0;
        let mut right_packages = Vec::with_capacity(checkpoint_count);
        for last_vpc_index in right_vpc_checkpoints {
            right_packages.push((start_vpc_index..=last_vpc_index).collect::<Vec<usize>>());
            start_vpc_index += 1;
        }

        left_packages.into_iter().rev().zip(right_packages.into_iter()).map(
            |(left_vpc_indices, right_vpc_indices)| {
                VpcIndexPackageDep { left_vpc_indices, right_vpc_indices }
            }
        ).collect()
    }
    // Return optimal vpc index of (left, right)
    pub fn get_optimal_position(
        left_vpc_vector: &Vec<Self>,
        right_vpc_vector: &Vec<Self>,
        anchor_scaled_penalty_delta: i64,
        anchor_size: u32,
    ) -> (usize, usize) {
        let mut optimal_left_vpc_index = 0;
        let mut optimal_right_vpc_index = 0;
        let mut optimal_max_query_length = 0;

        for (left_vpc_index, left_vpc) in left_vpc_vector.iter().enumerate().rev() {
            for (right_vpc_index, right_vpc) in right_vpc_vector.iter().enumerate().rev() {
                let scaled_penalty_delta = left_vpc.scaled_penalty_delta + right_vpc.scaled_penalty_delta + anchor_scaled_penalty_delta;

                if scaled_penalty_delta >= 0 {
                    let query_length = left_vpc.query_length + right_vpc.query_length + anchor_size;
                    if optimal_max_query_length < query_length {
                        optimal_max_query_length = query_length;
                        optimal_left_vpc_index = left_vpc_index;
                        optimal_right_vpc_index = right_vpc_index;
                    }
                    break
                }
            }
        }
        
        (optimal_left_vpc_index, optimal_right_vpc_index)
    }
}

#[cfg(test)]
mod tests {
    #[derive(Debug, Clone)]
    struct MyStruct {
        ql: usize,
        pd: usize,
    }

    #[test]
    fn print_testing_vpc_with_my_struct() {
        // let mut vector: Vec<MyStruct> = Vec::new();
        let mut vector: Vec<MyStruct> = vec![
            MyStruct { ql: 0, pd: 0 },
        ];

        let my_structs = vec![
            MyStruct { ql: 10, pd: 10 },
            MyStruct { ql: 12, pd: 6 },
            MyStruct { ql: 8, pd: 12 },
            MyStruct { ql: 30, pd: 60 },
            MyStruct { ql: 3, pd: 4 },
            MyStruct { ql: 14, pd: 20 },
            MyStruct { ql: 12, pd: 6 },
            MyStruct { ql: 30, pd: 50 },
            MyStruct { ql: 12, pd: 6 },
            MyStruct { ql: 32, pd: 40 },
            MyStruct { ql: 25, pd: 30 },
            MyStruct { ql: 18, pd: 5 },
        ];

        for my_struct in my_structs {
            println!("my_struct: {:?}", my_struct);
            let (ql, pd) = (my_struct.ql, my_struct.pd);

            let mut ql_index_to_insert: usize = 0;
            let mut pd_index_to_insert: usize = 0;
            let mut ql_is_same_as_pre = false;

            // Find index to insert
            for (index, my_struct_in_vector) in vector.iter().enumerate().rev() {
                // QL
                if ql_index_to_insert == 0 {
                    let checked_sub = ql.checked_sub(my_struct_in_vector.ql);
                    if let Some(gap) = checked_sub {
                        if gap == 0 {
                            ql_is_same_as_pre = true;
                        }
                        ql_index_to_insert = index + 1;
                    }
                }
                // PD
                if pd_index_to_insert == 0 {
                    if my_struct_in_vector.pd > pd {
                        pd_index_to_insert = index + 1;
                    }
                }
                if ql_index_to_insert != 0 && pd_index_to_insert != 0 {
                    break;
                }
            }

            println!("{}, {}", ql_index_to_insert, pd_index_to_insert);

            if ql_index_to_insert > pd_index_to_insert {
                // Delete middle elements and insert new
                (0..ql_index_to_insert-pd_index_to_insert).for_each(|_| {
                    vector.remove(pd_index_to_insert);
                });
                vector.insert(pd_index_to_insert, my_struct);
            } else if ql_index_to_insert == pd_index_to_insert {
                if !ql_is_same_as_pre {
                    if ql_index_to_insert == vector.len() {
                        vector.insert(pd_index_to_insert, my_struct);
                    } else {
                        if vector[ql_index_to_insert].pd < pd {
                            vector.insert(pd_index_to_insert, my_struct);
                        }
                    }
                }
            }

            println!("{:#?}", vector);
        }

        println!("{:#?}", vector);
    }
}


impl AnchorTable {
    #[inline]
    pub fn get_right_traversed_anchors(
        &self,
        anchor_index: &AnchorIndex,
        right_extension: &Extension,
        pattern_size: u32,
    ) -> Vec<AnchorIndex> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;
        let right_target_start_index = anchor_position.target_position + anchor_size;

        let mut traversed_anchors: Vec<AnchorIndex> = Vec::new(); // (pattern_index, target_position)

        // let right_extension = &local_extension.right_extension;
        let mut further_query_length = 0;
        let mut further_target_length = 0;

        for operations in right_extension.reversed_operations.iter().rev() {
            match operations.operation {
                AlignmentOperation::Match => {
                    let mut further_pattern_count = further_query_length / pattern_size;
                    let remained_length_from_previous_pattern_to_this_operations = further_query_length % pattern_size;
                    let length_to_next_pattern = if remained_length_from_previous_pattern_to_this_operations == 0 {
                        0
                    } else {
                        further_pattern_count += 1;
                        pattern_size - remained_length_from_previous_pattern_to_this_operations
                    };
                    // Traversed
                    if length_to_next_pattern + pattern_size <= operations.count {
                        let mut pattern_index = anchor_index.0 + pattern_count + further_pattern_count;
                        let mut target_position = right_target_start_index + further_target_length + length_to_next_pattern;
                        let anchor_index_in_pattern = loop {
                            let pattern_position = &self.0[pattern_index as usize];
                            let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, target_position);
                            match anchor_index_in_pattern {
                                Ok(index) => {
                                    break index as u32;
                                },
                                Err(_) => {
                                    pattern_index -= 1;
                                    target_position -= pattern_size;
                                },
                            }
                        };
                        traversed_anchors.push((pattern_index, anchor_index_in_pattern as u32));
                    }

                    further_query_length += operations.count;
                    further_target_length += operations.count;
                },
                AlignmentOperation::Subst => {
                    further_query_length += operations.count;
                    further_target_length += operations.count;
                },
                AlignmentOperation::Insertion => {
                    further_target_length += operations.count;
                },
                AlignmentOperation::Deletion => {
                    further_query_length += operations.count;
                },
            }
        }
        traversed_anchors
    }
    #[inline]
    pub fn get_left_traversed_anchors(
        &self,
        anchor_index: &AnchorIndex,
        local_extension: &LocalExtension,
        pattern_size: u32,
    ) -> Vec<AnchorIndex> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        let left_target_last_index = anchor_position.target_position;
        
        let mut traversed_anchors: Vec<AnchorIndex> = Vec::new(); // (pattern_index, target_position)

        let left_extension = &local_extension.left_extension;
        let mut further_query_length = 0;
        let mut further_target_length = 0;

        for operations in left_extension.reversed_operations.iter().rev() {
            match operations.operation {
                AlignmentOperation::Match => {
                    let mut further_pattern_count = further_query_length / pattern_size;
                    let remained_length_from_previous_pattern_to_this_operations = further_query_length % pattern_size;
                    let length_to_next_pattern = if remained_length_from_previous_pattern_to_this_operations == 0 {
                        0
                    } else {
                        further_pattern_count += 1;
                        pattern_size - remained_length_from_previous_pattern_to_this_operations
                    };
                    // Traversed
                    if length_to_next_pattern + pattern_size <= operations.count {
                        let mut pattern_index = anchor_index.0 - further_pattern_count;
                        let mut target_position = left_target_last_index - further_target_length - length_to_next_pattern;

                        let anchor_index_in_pattern = loop {
                            let pattern_position = &self.0[pattern_index.as_usize()];
                            let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, target_position);
                            match anchor_index_in_pattern {
                                Ok(index) => {
                                    break index as u32;
                                },
                                Err(_) => {
                                    pattern_index -= 1;
                                    target_position -= pattern_size;
                                },
                            }
                        };
                        traversed_anchors.push((pattern_index, anchor_index_in_pattern as u32));
                    }

                    further_query_length += operations.count;
                    further_target_length += operations.count;
                },
                AlignmentOperation::Subst => {
                    further_query_length += operations.count;
                    further_target_length += operations.count;
                },
                AlignmentOperation::Insertion => {
                    further_target_length += operations.count;
                },
                AlignmentOperation::Deletion => {
                    further_query_length += operations.count;
                },
            }
        }

        traversed_anchors
    }
}

 */