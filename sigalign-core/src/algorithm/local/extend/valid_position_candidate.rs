use crate::core::regulators::PREC_SCALE;
use super::{WaveFront, WaveFrontScore, BackTraceMarker};

// Validate Position Candidate
//  - Scalable penalty delta is i32:
//    PREC_SCALE is 100_000 now that the maximum query length is limited to about
//    2_147_483_647 (2^31) / (100_000 * MaxP) = 20_000 / MaxP
#[derive(Debug, Clone)]
pub struct Vpc {
    pub scaled_penalty_delta: i32,
    pub query_length: u32,
    pub penalty: u32,
    pub component_index: u32,
}

impl Vpc {
    // Return optimal vpc index of (left, right)
    pub fn get_optimal_position(
        left_sorted_vpc_vector: &Vec<Self>,
        right_sorted_vpc_vector: &Vec<Self>,
        anchor_scaled_penalty_delta: u32,
    ) -> (usize, usize) {
        let mut optimal_left_vpc_index = 0;
        let mut optimal_right_vpc_index = 0;
        let mut optimal_max_query_length_without_anchor = 0;

        for (left_vpc_index, left_vpc) in left_sorted_vpc_vector.iter().enumerate().rev() {
            for (right_vpc_index, right_vpc) in right_sorted_vpc_vector.iter().enumerate().rev() {
                let scaled_penalty_margin = left_vpc.scaled_penalty_delta + right_vpc.scaled_penalty_delta + anchor_scaled_penalty_delta as i32;

                if scaled_penalty_margin >= 0 {
                    let query_length_without_anchor = left_vpc.query_length + right_vpc.query_length;
                    if optimal_max_query_length_without_anchor < query_length_without_anchor {
                        optimal_max_query_length_without_anchor = query_length_without_anchor;
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

impl WaveFront {
    // Sorted by query length
    // --------------------------------
    // | QL |<QL |<QL |<QL | ... |<QL |
    // | PD>| PD>| PD>| PD>| ... | PD |
    // --------------------------------
    //TODO: Optimize
    #[inline]
    pub fn fill_sorted_vpc_vector(
        &self,
        maximum_penalty_per_scale: &u32,
        vpc_buffer: &mut Vec<Vpc>,
    ) {
        let last_penalty = self.end_point.penalty;

        self.wave_front_scores[..=last_penalty].iter().enumerate().for_each(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let scaled_penalty_delta = (length as u32 * maximum_penalty_per_scale) as i32 - (penalty * PREC_SCALE as usize) as i32;

            let mut ql_index_to_insert: usize = 0;
            let mut pd_index_to_insert: usize = 0;
            let mut ql_is_same_as_pre = false;

            // Find index to insert
            for (index, vpc_in_vector) in vpc_buffer.iter().enumerate().rev() {
                // QL
                if ql_index_to_insert == 0 {
                    let checked_sub = max_query_length.checked_sub(vpc_in_vector.query_length);
                    if let Some(gap) = checked_sub {
                        if gap == 0 {
                            ql_is_same_as_pre = true;
                        }
                        ql_index_to_insert = index + 1;
                    }
                }
                // PD
                if pd_index_to_insert == 0 && vpc_in_vector.scaled_penalty_delta > scaled_penalty_delta {
                    pd_index_to_insert = index + 1;
                }
                if ql_index_to_insert != 0 && pd_index_to_insert != 0 {
                    break;
                }
            }

            if ql_index_to_insert > pd_index_to_insert {
                // Delete middle elements and insert new
                (0..ql_index_to_insert-pd_index_to_insert).for_each(|_| {
                    vpc_buffer.remove(pd_index_to_insert);
                });
                vpc_buffer.insert(
                    pd_index_to_insert,
                    Vpc {
                        query_length: max_query_length,
                        scaled_penalty_delta,
                        penalty: penalty as u32,
                        component_index: comp_index,
                    },
                );
            } else if ql_index_to_insert == pd_index_to_insert {
                if !ql_is_same_as_pre {
                    if ql_index_to_insert == vpc_buffer.len() {
                        vpc_buffer.insert(
                            pd_index_to_insert,
                            Vpc {
                                query_length: max_query_length,
                                scaled_penalty_delta,
                                penalty: penalty as u32,
                                component_index: comp_index,
                            },
                        );
                    } else if vpc_buffer[ql_index_to_insert].scaled_penalty_delta < scaled_penalty_delta {
                        vpc_buffer.insert(
                            pd_index_to_insert,
                            Vpc {
                                query_length: max_query_length,
                                scaled_penalty_delta,
                                penalty: penalty as u32,
                                component_index: comp_index,
                            },
                        );
                    }
                }
            }
        });
    }
}

impl WaveFrontScore {
    // Result:
    //   (Maximum query index, Length of that, Component index of that)
    #[inline]
    fn point_of_maximum_query_length(&self) -> (u32, i32, u32) {
        let mut max_query_length = 0;
        let mut length_cache = 0;
        let mut comp_index_cache = 0;

        self.components_by_k.iter().enumerate().for_each(|(comp_index, comp)| {
            if comp.m.bt != BackTraceMarker::Empty {
                let query_length = comp.m.fr + self.max_k - comp_index as i32; // Fr - k
                if max_query_length < query_length {
                    max_query_length = query_length;
                    length_cache = comp.m.fr + comp.m.insertion_count as i32;
                    comp_index_cache = comp_index;
                }
            }
        });

        (max_query_length as u32, length_cache, comp_index_cache as u32)
    }
}
