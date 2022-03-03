use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

// Validate Position Candidate
#[derive(Debug, Clone)]
pub struct VPC {
    pub query_length: usize,
    pub penalty: usize,
    pub component_index: usize,
    pub scaled_penalty_margin: i64,
}

impl VPC {
    // Return optimal vpc index of (left, right)
    pub fn get_optimal_position(
        left_vpc_vector: &Vec<Self>,
        right_vpc_vector: &Vec<Self>,
        anchor_scaled_penalty_margin: i64,
        anchor_size: usize,
    ) -> (usize, usize) {
        let mut optimal_left_vpc_index = 0;
        let mut optimal_right_vpc_index = 0;
        let mut optimal_max_query_length = 0;

        for (left_vpc_index, left_vpc) in left_vpc_vector.iter().enumerate().rev() {
            for (right_vpc_index, right_vpc) in right_vpc_vector.iter().enumerate().rev() {
                let scaled_penalty_margin = left_vpc.scaled_penalty_margin + right_vpc.scaled_penalty_margin + anchor_scaled_penalty_margin;

                if scaled_penalty_margin >= 0 {
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

impl WaveFront {
    // Sorted by query length
    // --------------------------------
    // | QL |<QL |<QL |<QL | ... |<QL |
    // | PM>| PM>| PM>| PM>| ... | PM |
    // --------------------------------
    pub fn get_sorted_vpc_vector(&self, maximum_penalty_per_scale: usize, minimum_scaled_penalty_margin: i64) -> Vec<VPC> {
        let last_score = self.end_point.score;

        let mut sorted_vpc_vector: Vec<VPC> = Vec::new();

        self.wave_front_scores[..=last_score].iter().enumerate().for_each(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let scaled_penalty_margin = (length as usize * maximum_penalty_per_scale) as i64 - (penalty * PRECISION_SCALE) as i64;

            if minimum_scaled_penalty_margin <= scaled_penalty_margin {
                let mut ql_index_to_insert: usize = 0;
                let mut pm_index_to_insert: usize = 0;
                let mut ql_is_same_as_pre = false;

                // Find index to insert
                for (index, vpc_in_vector) in sorted_vpc_vector.iter().enumerate().rev() {
                    // QL
                    if ql_index_to_insert == 0 {
                        let checked_sub = max_query_length.checked_sub(vpc_in_vector.query_length as usize);
                        if let Some(gap) = checked_sub {
                            if gap == 0 {
                                ql_is_same_as_pre = true;
                            }
                            ql_index_to_insert = index + 1;
                        }
                    }
                    // PM
                    if pm_index_to_insert == 0 {
                        if vpc_in_vector.scaled_penalty_margin > scaled_penalty_margin {
                            pm_index_to_insert = index + 1;
                        }
                    }
                    if ql_index_to_insert != 0 && pm_index_to_insert != 0 {
                        break;
                    }
                }

                if ql_index_to_insert > pm_index_to_insert {
                    // Delete middle elements and insert new
                    (0..ql_index_to_insert-pm_index_to_insert).for_each(|_| {
                        sorted_vpc_vector.remove(pm_index_to_insert);
                    });
                    sorted_vpc_vector.insert(
                        pm_index_to_insert,
                        VPC {
                            query_length: max_query_length,
                            scaled_penalty_margin,
                            penalty: penalty,
                            component_index: comp_index,
                        },
                    );
                } else if ql_index_to_insert == pm_index_to_insert {
                    if !ql_is_same_as_pre {
                        if ql_index_to_insert == sorted_vpc_vector.len() {
                            sorted_vpc_vector.insert(
                                pm_index_to_insert,
                                VPC {
                                    query_length: max_query_length,
                                    scaled_penalty_margin,
                                    penalty: penalty,
                                    component_index: comp_index,
                                },
                            );
                        } else {
                            if sorted_vpc_vector[ql_index_to_insert].scaled_penalty_margin < scaled_penalty_margin {
                                sorted_vpc_vector.insert(
                                    pm_index_to_insert,
                                    VPC {
                                        query_length: max_query_length,
                                        scaled_penalty_margin,
                                        penalty: penalty,
                                        component_index: comp_index,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        });
            
        sorted_vpc_vector
    }
}

impl WaveFrontScore {
    fn point_of_maximum_query_length(&self) -> (usize, i32, usize) { // (Maximum query index, Length of that, Component index of that)
        let mut max_query_length = 0;
        let mut length_cache = 0;
        let mut comp_index_cache = 0;

        self.components_by_k.iter().enumerate().for_each(|(comp_index, comp)| {
            if comp.m.bt != BackTraceMarker::Empty {
                let query_length = comp.m.fr + self.max_k - comp_index as i32; // Fr - k
                if max_query_length < query_length {
                    max_query_length = query_length;
                    length_cache = comp.m.fr + comp.m.deletion_count as i32;
                    comp_index_cache = comp_index;
                }
            }
        });

        (max_query_length as usize, length_cache, comp_index_cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MyStruct {
        ql: usize,
        pm: usize,
    }

    #[test]
    fn print_testing_vpc_with_my_struct() {
        // let mut vector: Vec<MyStruct> = Vec::new();
        let mut vector: Vec<MyStruct> = vec![
            MyStruct { ql: 0, pm: 0 },
        ];

        let my_structs = vec![
            MyStruct { ql: 10, pm: 10 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 8, pm: 12 },
            MyStruct { ql: 30, pm: 60 },
            MyStruct { ql: 3, pm: 4 },
            MyStruct { ql: 14, pm: 20 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 30, pm: 50 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 32, pm: 40 },
            MyStruct { ql: 25, pm: 30 },
            MyStruct { ql: 18, pm: 5 },
        ];

        for my_struct in my_structs {
            println!("my_struct: {:?}", my_struct);
            let (ql, pm) = (my_struct.ql, my_struct.pm);

            let mut ql_index_to_insert: usize = 0;
            let mut pm_index_to_insert: usize = 0;
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
                // PM
                if pm_index_to_insert == 0 {
                    if my_struct_in_vector.pm > pm {
                        pm_index_to_insert = index + 1;
                    }
                }
                if ql_index_to_insert != 0 && pm_index_to_insert != 0 {
                    break;
                }
            }

            println!("{}, {}", ql_index_to_insert, pm_index_to_insert);

            if ql_index_to_insert > pm_index_to_insert {
                // Delete middle elements and insert new
                (0..ql_index_to_insert-pm_index_to_insert).for_each(|_| {
                    vector.remove(pm_index_to_insert);
                });
                vector.insert(pm_index_to_insert, my_struct);
            } else if ql_index_to_insert == pm_index_to_insert {
                if !ql_is_same_as_pre {
                    if ql_index_to_insert == vector.len() {
                        vector.insert(pm_index_to_insert, my_struct);
                    } else {
                        if vector[ql_index_to_insert].pm < pm {
                            vector.insert(pm_index_to_insert, my_struct);
                        }
                    }
                }
            }

            println!("{:#?}", vector);
        }

        println!("{:#?}", vector);
    }
}
