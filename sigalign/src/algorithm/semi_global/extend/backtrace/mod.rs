use crate::{
    core::regulators::Penalty,
};
use super::{
    AnchorIndex,
    WaveFront, BackTraceMarker,
};
use num::integer::div_rem;

enum ComponentType {
    M,
    I,
    D,
}

// TODO: Backtrace can refer the other extensions of this anchor
impl WaveFront {
    #[inline]
    pub fn backtrace_to_get_left_side_traversed_anchor(
        &self,
        mut penalty: u32,
        pattern_size: u32,
        component_index: u32,
        penalties: &Penalty,
        traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    ) -> (u32, u32) {
        let traversed_anchor_start_index = traversed_anchor_index_buffer.len() as u32;
        
        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

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
                                let estimated_additive_position_info = (
                                    quotient as u32,
                                    (next_fr + match_count_of_next_pattern + 1) as u32,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            
                            // (8) Add operation
                            //  - skip
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
                                let estimated_additive_position_info = (
                                    quotient as u32,
                                    (next_fr + match_count_of_next_pattern) as u32,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            // (8) Add operation
                            //  - skip
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
                                let estimated_additive_position_info = (
                                    quotient as u32,
                                    (next_fr + match_count_of_next_pattern) as u32,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            // (8) Add operation
                            //  - skip
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            // Add operation
                            //  - skip
                            return (
                                traversed_anchor_start_index,
                                traversed_anchor_index_buffer.len() as u32,
                            );
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
                            //  - skip
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
                            //  - skip
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
                            //  - skip
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
                            //  - skip
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
    #[inline]
    pub fn backtrace_to_get_right_side_traversed_anchor(
        &self,
        mut penalty: u32,
        pattern_size: u32,
        pattern_count_of_anchor: u32,
        component_index: u32,
        penalties: &Penalty,
        traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    ) -> (u32, u32) { // Result is traversed anchor range
        let traversed_anchor_start_index = traversed_anchor_index_buffer.len() as u32;

        let wave_front_scores = &self.wave_front_scores;

        // Initialize
        let anchor_size: u32 = pattern_count_of_anchor * pattern_size;

        let mut wave_front_score = &wave_front_scores[penalty as usize];

        let mut component_type = ComponentType::M;
        let mut component = &wave_front_score.components_by_k[component_index as usize].m;
        let mut k = -wave_front_score.max_k + component_index as i32;
        let mut fr = component.fr;

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
                                let estimated_additive_position_info = (
                                    (quotient + 1) as u32 + pattern_count_of_anchor,
                                    (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            
                            // (8) Add operation
                            //  - skip
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
                                let estimated_additive_position_info = (
                                    (quotient + 1) as u32 + pattern_count_of_anchor,
                                    (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            // (8) Add operation
                            //  - skip
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
                                let estimated_additive_position_info = (
                                    (quotient + 1) as u32 + pattern_count_of_anchor,
                                    (fr - match_count_of_next_pattern) as u32 + anchor_size,
                                );
                                traversed_anchor_index_buffer.push(estimated_additive_position_info);
                            }
                            // (8) Add operation
                            //  - skip
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                        _ => { // START_POINT
                            // Add operation
                            //  - skip
                            return (
                                traversed_anchor_start_index,
                                traversed_anchor_index_buffer.len() as u32,
                            );
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
                            //  - skip
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
                            //  - skip
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
                            //  - skip
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
                            //  - skip
                            // (9) Next fr to fr
                            fr = next_fr;
                        },
                    }
                },
            };
        }
    }
}
