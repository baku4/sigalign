use crate::{
    core::regulators::{Penalty, PREC_SCALE},
    results::{
        AlignmentOperation, AlignmentOperations,
    },
};
use super::{
    WaveFront, BackTraceMarker,
};
use num::integer::div_rem;

pub struct BackTraceResult {
    pub operation_buffer_range: (u32, u32), // start, end
    pub processed_length: (u32, u32), // query, target
    pub length_of_extension: u32,
    pub penalty_of_extension: u32,
}

enum ComponentType {
    M,
    I,
    D,
}

#[derive(Debug, Clone)]
pub struct TraversedAnchor {
    pub addt_pattern_index: u32,
    pub addt_target_position: u32,
    pub cum_penalty_delta: i32,
    pub to_skip: bool,
}

impl WaveFront {
    #[inline]
    // When the traversed anchor is detected in the left, return None immediately.
    // TODO: Refine the variables' name
    pub fn backtrace_of_left_side(
        &self,
        mut penalty: u32,
        pattern_size: u32,
        component_index: u32,
        penalties: &Penalty,
        operations_buffer: &mut Vec<AlignmentOperations>,
    ) -> Option<BackTraceResult> {
        operations_buffer.push(AlignmentOperations {
            operation: AlignmentOperation::Deletion,
            count: 0,
        });
        let operation_start_index = operations_buffer.len() as u32;
        
        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let total_penalty = penalty;
        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

        let total_length = fr as u32 + component.insertion_count as u32;
        let total_processed_query = (fr - k) as u32;
        let total_processed_target = fr as u32;

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
                                // Drop it
                                return None
                            }
                            
                            // (8) Add operation
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: last_fr
                                    }) = operations_buffer.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations_buffer.push(
                                        AlignmentOperations {
                                            operation: AlignmentOperation::Subst,
                                            count: 1,
                                        }
                                    );
                                }
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
                                    }
                                );
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: 1,
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

                            let (quotient, remainder) = div_rem(fr - k, pattern_size as i32);
                            let match_count_of_next_pattern = match_count - remainder;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                // Drop it
                                return None
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
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
                            let match_count = fr-next_fr;

                            let (quotient, remainder) = div_rem(fr - k, pattern_size as i32);
                            let match_count_of_next_pattern = match_count - remainder;
                            if match_count_of_next_pattern >= pattern_size as i32 {
                                // Drop it
                                return None
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations_buffer.push(
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
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: fr as u32,
                                    }
                                );
                            };

                            let backtrace_result = BackTraceResult {
                                // start, size
                                operation_buffer_range: (
                                    operation_start_index,
                                    operations_buffer.len() as u32,
                                ),
                                // query, target
                                processed_length: (total_processed_query, total_processed_target),
                                length_of_extension: total_length,
                                penalty_of_extension: total_penalty,
                            };
                            return Some(backtrace_result);
                        }
                    }
                },
                /* I */
                ComponentType::D => {
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
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
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
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
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
                /* D */
                ComponentType::I => {
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
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
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
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
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
            };
        }
    }
    #[inline]
    pub fn backtrace_of_right_side(
        &self,
        mut penalty: u32,
        scaled_maximum_penalty_per_length: i32,
        pattern_size: u32,
        pattern_count_of_anchor: u32,
        component_index: u32,
        penalties: &Penalty,
        operations_buffer: &mut Vec<AlignmentOperations>,
        traversed_anchor_status_buffer: &mut Vec<TraversedAnchor>,
    ) -> BackTraceResult {
        traversed_anchor_status_buffer.clear();

        operations_buffer.push(AlignmentOperations {
            operation: AlignmentOperation::Deletion,
            count: 0,
        });
        let operation_start_index = operations_buffer.len() as u32;
        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let anchor_size: u32 = pattern_count_of_anchor * pattern_size;

        let total_penalty = penalty;
        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

        let total_length = fr as u32 + component.insertion_count as u32 + anchor_size;
        let total_processed_query = (fr - k) as u32 + anchor_size;
        let total_processed_target = fr as u32 + anchor_size;

        // Penalty delta from start point to the traversed matches
        //   - traversed matches: consecutive match containing traversed anchor
        let mut pd_to_previous_tv_matches = 0;

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
                            // two base before the match block (*: want, +: match block)
                            //      --------
                            //      MMMMSMMM
                            //      --------
                            //         ^ ^
                            //         * +
                            //         |
                            //         next_fr - k
                            let length_to_two_base_before_match_block: i32 = next_fr - k;
                            
                            let (quotient, remainder) = div_rem(length_to_two_base_before_match_block, pattern_size as i32);
                            let match_count_of_assumed_anchor = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_assumed_anchor >= pattern_size as i32 {
                                // Traversed Anchor Exists
                                let pd_to_this_tv_matches = (
                                    scaled_maximum_penalty_per_length
                                    * (next_fr + component.insertion_count as i32 + 1) // Length
                                ) - (
                                    (penalty + penalties.x) * PREC_SCALE // Penalty
                                ) as i32;
                                let pd_between_tv_matches = pd_to_previous_tv_matches - pd_to_this_tv_matches;
                                traversed_anchor_status_buffer.iter_mut().for_each(|tv| {
                                    tv.cum_penalty_delta += pd_between_tv_matches;
                                    if tv.cum_penalty_delta > 0 {
                                        tv.to_skip = true;
                                    }
                                });
                                pd_to_previous_tv_matches = pd_to_this_tv_matches;
                                let traversed_anchor = TraversedAnchor {
                                    addt_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    addt_target_position: (fr - match_count_of_assumed_anchor) as u32 + anchor_size,
                                    cum_penalty_delta: 0,
                                    to_skip: false
                                };
                                traversed_anchor_status_buffer.push(traversed_anchor);
                            }
                            
                            // (8) Add operation
                            if match_count == 0 {
                                if let Some(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: last_fr
                                    }) = operations_buffer.last_mut() {
                                    *last_fr += 1;
                                } else {
                                    operations_buffer.push(
                                        AlignmentOperations {
                                            operation: AlignmentOperation::Subst,
                                            count: 1
                                        }
                                    );
                                }
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32
                                    }
                                );
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Subst,
                                        count: 1
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
                            // two base before the match block (*: want, +: match block)
                            //      --------
                            //      MMMMDMMM
                            //      ---- ---
                            //        ^^ ^
                            //        *| +
                            //         |
                            //         next_fr - k
                            let length_to_two_base_before_match_block = next_fr - k - 1;

                            let (quotient, remainder) = div_rem(length_to_two_base_before_match_block, pattern_size as i32);
                            let match_count_of_assumed_anchor = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_assumed_anchor >= pattern_size as i32 {
                                // Traversed Anchor Exists
                                let pd_to_this_tv_matches = 
                                    scaled_maximum_penalty_per_length
                                    * (next_fr + component.insertion_count as i32) // Length
                                    - (penalty * PREC_SCALE) as i32 // Penalty
                                ;
                                let pd_between_tv_matches = pd_to_previous_tv_matches - pd_to_this_tv_matches;
                                traversed_anchor_status_buffer.iter_mut().for_each(|tv| {
                                    tv.cum_penalty_delta += pd_between_tv_matches;
                                    if tv.cum_penalty_delta > 0 {
                                        tv.to_skip = true;
                                    }
                                });
                                pd_to_previous_tv_matches = pd_to_this_tv_matches;
                                let traversed_anchor = TraversedAnchor {
                                    addt_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    addt_target_position: (fr - match_count_of_assumed_anchor) as u32 + anchor_size,
                                    cum_penalty_delta: 0,
                                    to_skip: false
                                };
                                traversed_anchor_status_buffer.push(traversed_anchor);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Match,
                                        count: match_count as u32,
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
                            // two base before the match block (*: want, +: match block)
                            //      ---- ---
                            //      MMMMIMMM
                            //      --------
                            //         ^^^
                            //         *|+
                            //          next_fr - k
                            let length_to_two_base_before_match_block = next_fr - k - 1;

                            let (quotient, remainder) = div_rem(length_to_two_base_before_match_block, pattern_size as i32);
                            let match_count_of_assumed_anchor = match_count + remainder + 1 - pattern_size as i32;
                            if match_count_of_assumed_anchor >= pattern_size as i32 {
                                // Traversed Anchor Exists
                                let pd_to_this_tv_matches = 
                                    scaled_maximum_penalty_per_length
                                    * (next_fr + component.insertion_count as i32) // Length
                                    - (penalty * PREC_SCALE) as i32 // Penalty
                                ;
                                let pd_between_tv_matches = pd_to_previous_tv_matches - pd_to_this_tv_matches;
                                traversed_anchor_status_buffer.iter_mut().for_each(|tv| {
                                    tv.cum_penalty_delta += pd_between_tv_matches;
                                    if tv.cum_penalty_delta > 0 {
                                        tv.to_skip = true;
                                    }
                                });
                                pd_to_previous_tv_matches = pd_to_this_tv_matches;
                                let traversed_anchor = TraversedAnchor {
                                    addt_pattern_index: (quotient + 1) as u32 + pattern_count_of_anchor,
                                    addt_target_position: (fr - match_count_of_assumed_anchor) as u32 + anchor_size,
                                    cum_penalty_delta: 0,
                                    to_skip: false
                                };
                                traversed_anchor_status_buffer.push(traversed_anchor);
                            }
                            // (8) Add operation
                            if match_count != 0 {
                                operations_buffer.push(
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
                            operations_buffer.push(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Match,
                                    count: last_match_count,
                                }
                            );
                            let pd_between_tv_matches = 
                                pd_to_previous_tv_matches
                                + (anchor_size as i32 * scaled_maximum_penalty_per_length)
                                // pd after anchor + pd of anchor
                            ;
                            traversed_anchor_status_buffer.iter_mut().for_each(|tv| {
                                tv.cum_penalty_delta += pd_between_tv_matches;
                                if tv.cum_penalty_delta > 0 {
                                    tv.to_skip = true;
                                }
                            });

                            let backtrace_result = BackTraceResult {
                                // start, size
                                operation_buffer_range: (
                                    operation_start_index,
                                    operations_buffer.len() as u32,
                                ),
                                // query, target
                                processed_length: (total_processed_query, total_processed_target),
                                length_of_extension: total_length,
                                penalty_of_extension: total_penalty,
                            };
                            return backtrace_result;
                        }
                    }
                },
                /* I */
                ComponentType::D => {
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
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Deletion,
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
                            component = wave_front_score.d_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Deletion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
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
                /* D */
                ComponentType::I => {
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
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
                                    AlignmentOperations {
                                        operation: AlignmentOperation::Insertion,
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
                            component = wave_front_score.i_component_of_k(k);
                            // (6) Next fr
                            let next_fr = component.fr;
                            // (7) Add operation
                            if let Some(
                                AlignmentOperations {
                                    operation: AlignmentOperation::Insertion,
                                    count: last_fr
                                }) = operations_buffer.last_mut() {
                                *last_fr += 1;
                            } else {
                                operations_buffer.push(
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
            };
        }
    }
}
