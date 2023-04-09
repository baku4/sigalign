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
use super::{PosTable, AnchorIndex, AnchorPosition, TraversedAnchor};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use super::LocalExtension;
use ahash::AHashSet;


// Validate Position Candidate
#[derive(Debug, Clone)]
pub struct Vpc {
    pub scaled_penalty_delta: i64,
    pub query_length: u32,
    pub penalty: u32,
    pub component_index: u32,
}

impl WaveFront {
    #[inline]
    pub fn backtrace_for_local(
        &self,
        maximum_penalty_per_scale: &u32,
        penalties: &Penalty,
        sorted_vpc_vector_buffer: &mut Vec<Vpc>,
    ) {
        // Clear buffer here
        sorted_vpc_vector_buffer.clear();

        // Fill vpc vector
        self.fill_sorted_vpc_vector(maximum_penalty_per_scale, sorted_vpc_vector_buffer);

        // Parse extensions
        sorted_vpc_vector_buffer.iter().for_each(|vpc| {
            let extension = self.backtrace_from_endpoint(
                vpc.penalty,
                vpc.component_index,
                penalties,
            );
        })
    }
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
        sorted_vpc_vector_buffer: &mut Vec<Vpc>,
    ) {
        let last_penalty = self.end_point.penalty;

        self.wave_front_scores[..=last_penalty].iter().enumerate().for_each(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let scaled_penalty_delta = (length as u32 * maximum_penalty_per_scale) as i64 - (penalty * PREC_SCALE as usize) as i64;

            let mut ql_index_to_insert: usize = 0;
            let mut pd_index_to_insert: usize = 0;
            let mut ql_is_same_as_pre = false;

            // Find index to insert
            for (index, vpc_in_vector) in sorted_vpc_vector_buffer.iter().enumerate().rev() {
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
                if pd_index_to_insert == 0 {
                    if vpc_in_vector.scaled_penalty_delta > scaled_penalty_delta {
                        pd_index_to_insert = index + 1;
                    }
                }
                if ql_index_to_insert != 0 && pd_index_to_insert != 0 {
                    break;
                }
            }

            if ql_index_to_insert > pd_index_to_insert {
                // Delete middle elements and insert new
                (0..ql_index_to_insert-pd_index_to_insert).for_each(|_| {
                    sorted_vpc_vector_buffer.remove(pd_index_to_insert);
                });
                sorted_vpc_vector_buffer.insert(
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
                    if ql_index_to_insert == sorted_vpc_vector_buffer.len() {
                        sorted_vpc_vector_buffer.insert(
                            pd_index_to_insert,
                            Vpc {
                                query_length: max_query_length,
                                scaled_penalty_delta,
                                penalty: penalty as u32,
                                component_index: comp_index,
                            },
                        );
                    } else {
                        if sorted_vpc_vector_buffer[ql_index_to_insert].scaled_penalty_delta < scaled_penalty_delta {
                            sorted_vpc_vector_buffer.insert(
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
                    length_cache = comp.m.fr + comp.m.deletion_count as i32;
                    comp_index_cache = comp_index;
                }
            }
        });

        (max_query_length as u32, length_cache, comp_index_cache as u32)
    }
}

impl WaveFront {
    #[inline]
    pub fn get_sorted_vpc_vector_dep(
        &self,
        maximum_penalty_per_scale: u32,
        minimum_scaled_penalty_delta: i64,
    ) -> Vec<Vpc> {
        let last_penalty = self.end_point.penalty;

        let mut sorted_vpc_vector: Vec<Vpc> = Vec::new();

        self.wave_front_scores[..=last_penalty].iter().enumerate().for_each(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let scaled_penalty_delta = (length as u32 * maximum_penalty_per_scale) as i64 - (penalty * PREC_SCALE as usize) as i64;

            if minimum_scaled_penalty_delta <= scaled_penalty_delta {
                let mut ql_index_to_insert: usize = 0;
                let mut pd_index_to_insert: usize = 0;
                let mut ql_is_same_as_pre = false;

                // Find index to insert
                for (index, vpc_in_vector) in sorted_vpc_vector.iter().enumerate().rev() {
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
                    if pd_index_to_insert == 0 {
                        if vpc_in_vector.scaled_penalty_delta > scaled_penalty_delta {
                            pd_index_to_insert = index + 1;
                        }
                    }
                    if ql_index_to_insert != 0 && pd_index_to_insert != 0 {
                        break;
                    }
                }

                if ql_index_to_insert > pd_index_to_insert {
                    // Delete middle elements and insert new
                    (0..ql_index_to_insert-pd_index_to_insert).for_each(|_| {
                        sorted_vpc_vector.remove(pd_index_to_insert);
                    });
                    sorted_vpc_vector.insert(
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
                        if ql_index_to_insert == sorted_vpc_vector.len() {
                            sorted_vpc_vector.insert(
                                pd_index_to_insert,
                                Vpc {
                                    query_length: max_query_length,
                                    scaled_penalty_delta,
                                    penalty: penalty as u32,
                                    component_index: comp_index,
                                },
                            );
                        } else {
                            if sorted_vpc_vector[ql_index_to_insert].scaled_penalty_delta < scaled_penalty_delta {
                                sorted_vpc_vector.insert(
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
                }
            }
        });
            
        sorted_vpc_vector
    }
}

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


impl PosTable {
    #[inline]
    pub fn get_right_traversed_anchors(
        &self,
        anchor_index: &AnchorIndex,
        local_extension: &LocalExtension,
        pattern_size: u32,
    ) -> Vec<AnchorIndex> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;
        let right_target_start_index = anchor_position.position_in_target + anchor_size;

        let mut traversed_anchors: Vec<AnchorIndex> = Vec::new(); // (pattern_index, target_position)

        let right_extension = &local_extension.right_extension;
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

        let left_target_last_index = anchor_position.position_in_target;
        
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