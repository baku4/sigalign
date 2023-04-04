use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation,
    }
};

use super::{PosTable, AnchorIndex, AnchorPosition, TraversedAnchor};
use super::{Extension, WaveFront, calculate_spare_penalty};
use super::{Vpc, VpcIndexPackage};

#[derive(Debug, Clone)]
pub struct LocalExtensionDep {
    pub left_extension: Extension,
    pub right_extension: Extension,
    pub left_traversed_anchors: Vec<TraversedAnchor>,
    pub right_traversed_anchors: Vec<TraversedAnchor>,
    pub left_scaled_penalty_deltas: Vec<i64>,
    pub right_scaled_penalty_deltas: Vec<i64>,
}
#[derive(Debug, Clone)]
pub struct LocalExtension {
    pub left_extension: Extension,
    pub right_extension: Extension,
    pub left_checkpoints: Vec<(u32, u32)>,
    pub right_checkpoints: Vec<(u32, u32)>,
    pub representative_anchor: AnchorIndex,
}

impl PosTable {
    #[inline]
    pub fn extend_assuming_leftmost_anchor_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_left: &i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> Vec<LocalExtension> { // (Vec of LocalExtension, Right traversed anchors)
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_target_last_index = anchor_position.position_in_target;
        let right_target_start_index = left_target_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = anchor_size.as_i64() * cutoff.maximum_penalty_per_scale as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_target_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let right_spare_penalty = calculate_spare_penalty(*scaled_penalty_delta_of_left, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_left;
        let right_sorted_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_delta);

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_target_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let right_max_scaled_penalty_delta = right_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_delta, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - right_max_scaled_penalty_delta;
        let left_sorted_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_delta);

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );

        //
        // (5) Get extension results from each Vpc index package
        //
        let local_extensions: Vec<LocalExtension> = {
            vpc_index_packages.into_iter().map(| VpcIndexPackage {
                left_vpc_indices,
                right_vpc_indices,
            } | {
                let left_checkpoints: Vec<(u32, u32)> = left_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &left_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = left_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = left_query_last_index - query_extended_length as u32;
                    let target_position = left_target_last_index - target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let right_checkpoints: Vec<(u32, u32)> = right_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &right_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = right_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = right_query_start_index + query_extended_length as u32;
                    let target_position = right_target_start_index + target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let left_vpc = &left_sorted_vpc_vector[*unsafe { left_vpc_indices.last().unwrap_unchecked() }];
                let left_extension = left_wave_front.backtrace_from_endpoint(left_vpc.penalty, left_vpc.component_index, penalties);
                let right_vpc = &right_sorted_vpc_vector[*unsafe { right_vpc_indices.last().unwrap_unchecked() }];
                let right_extension = right_wave_front.backtrace_from_endpoint(right_vpc.penalty, right_vpc.component_index, penalties);
                LocalExtension {
                    left_checkpoints,
                    right_checkpoints,
                    left_extension,
                    right_extension,
                    representative_anchor: *anchor_index,
                }
            }).collect()
        };

        local_extensions
    }
    #[inline]
    pub fn extend_assuming_rightmost_anchor_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_right: &i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> Vec<LocalExtension> {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_target_last_index = anchor_position.position_in_target;
        let right_target_start_index = left_target_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_target_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let left_spare_penalty = calculate_spare_penalty(*scaled_penalty_delta_of_right, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_right;
        let left_sorted_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_delta);

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_target_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let left_max_scaled_penalty_delta = left_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_delta, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - left_max_scaled_penalty_delta;
        let right_sorted_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_delta);

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );

        //
        // (5) Get extension results from each Vpc index package
        //
        let local_extensions: Vec<LocalExtension> = {
            vpc_index_packages.into_iter().map(| VpcIndexPackage {
                left_vpc_indices,
                right_vpc_indices,
            } | {
                let left_checkpoints: Vec<(u32, u32)> = left_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &left_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = left_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = left_query_last_index - query_extended_length as u32;
                    let target_position = left_target_last_index - target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let right_checkpoints: Vec<(u32, u32)> = right_vpc_indices.iter().map(|vpc_index| {
                    let vpc = &right_sorted_vpc_vector[*vpc_index];
                    let (query_extended_length, target_extended_length) = right_wave_front.get_extended_length_of_endpoint(vpc.penalty, vpc.component_index);
                    let query_position = right_query_start_index + query_extended_length as u32;
                    let target_position = right_target_start_index + target_extended_length as u32;
                    (query_position, target_position)
                }).collect();
                let left_vpc = &left_sorted_vpc_vector[*unsafe { left_vpc_indices.last().unwrap_unchecked() }];
                let left_extension = left_wave_front.backtrace_from_endpoint(left_vpc.penalty, left_vpc.component_index, penalties);
                let right_vpc = &right_sorted_vpc_vector[*unsafe { right_vpc_indices.last().unwrap_unchecked() }];
                let right_extension = right_wave_front.backtrace_from_endpoint(right_vpc.penalty, right_vpc.component_index, penalties);
                LocalExtension {
                    left_checkpoints,
                    right_checkpoints,
                    left_extension,
                    right_extension,
                    representative_anchor: *anchor_index,
                }
            }).collect()
        };

        //
        // (6) Get left traversed anchors
        //
        let left_traversed_anchors = {
            let leftmost_local_extension = &local_extensions[0];
            let mut traversed_anchors: Vec<AnchorIndex> = Vec::new(); // (pattern_index, target_position)

            let left_extension = &leftmost_local_extension.left_extension;
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
                            let pattern_index = anchor_index.0 - further_pattern_count;
                            let target_position = left_target_last_index - further_target_length - length_to_next_pattern;
                            let pattern_position = &self.0[pattern_index as usize];
                            let anchor_index_in_pattern = unsafe { AnchorPosition::binary_search_index(pattern_position, target_position).unwrap_unchecked() };
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
        };

        // (local_extensions, left_traversed_anchors)
        local_extensions
    }
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
                        let pattern_index = anchor_index.0 + pattern_count + further_pattern_count;
                        let target_position = right_target_start_index + further_target_length + length_to_next_pattern;
                        let pattern_position = &self.0[pattern_index as usize];
                        let anchor_index_in_pattern = unsafe { AnchorPosition::binary_search_index(pattern_position, target_position).unwrap_unchecked() };
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
                        let pattern_index = anchor_index.0 - further_pattern_count;
                        let target_position = left_target_last_index - further_target_length - length_to_next_pattern;
                        let pattern_position = &self.0[pattern_index as usize];
                        let anchor_index_in_pattern = unsafe { AnchorPosition::binary_search_index(pattern_position, target_position).unwrap_unchecked() };
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
    pub fn extend_right_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target_sequence: &[u8],
        query_sequence: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_left: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtensionDep {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.position_in_target;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = anchor_size.as_i64() * cutoff.maximum_penalty_per_scale as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &target_sequence[right_record_start_index as usize..];
        let right_query_slice = &query_sequence[right_query_start_index as usize..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_left, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_left;
        let right_sorted_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_delta);

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &target_sequence[..left_record_last_index as usize];
        let left_query_slice = &query_sequence[..left_query_last_index as usize];

        let right_max_scaled_penalty_delta = right_sorted_vpc_vector[0].scaled_penalty_delta as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_delta, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - right_max_scaled_penalty_delta;
        let left_sorted_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_delta);

        //
        // (4) Get packaged indices of VPC vector
        // The result is sorted from left to right
        //
        let vpc_index_packages = Vpc::package_vpc_index(
            &left_sorted_vpc_vector,
            &right_sorted_vpc_vector,
            anchor_scaled_penalty_delta,
        );

        //
        // (5) Get extension result from each Vpc index package
        //
        // let left_traversed_anchors = Vec::new();
        // let right_traversed_anchors = Vec::new();
        // let local_extensions = {
        //     vpc_index_packages.into_iter().map(| VpcIndexPackage {
        //         left_vpc_indices,
        //         right_vpc_indices,
        //     } | {
        //         // let left_extension = left_wave_front.backtrace_from_endpoint(penalty, index_of_component, penalties)
        //     })
        // };

        let (left_vpc_index, right_vpc_index) = Vpc::get_optimal_position(&left_sorted_vpc_vector, &right_sorted_vpc_vector, anchor_scaled_penalty_delta, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_sorted_vpc_vector[left_vpc_index];
        let right_vpc = &right_sorted_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty delta
        //
        let left_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &left_extension,
            &left_sorted_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &right_extension,
            &right_sorted_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtensionDep {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_deltas,
            right_scaled_penalty_deltas,
        }
    }
    #[inline]
    pub fn extend_left_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_right: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtensionDep {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.position_in_target;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_delta = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &target[..left_record_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_right, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - scaled_penalty_delta_of_right;
        let left_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_delta);

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &target[right_record_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let left_max_scaled_penalty_delta = left_vpc_vector[0].scaled_penalty_delta as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_delta, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_delta = - anchor_scaled_penalty_delta - left_max_scaled_penalty_delta;
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_delta);

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = Vpc::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_delta, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty delta
        //
        let left_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &left_extension,
            &left_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_deltas: Vec<i64> = get_scaled_penalty_deltas_of_vpc_vector(
            &right_extension,
            &right_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtensionDep {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_deltas,
            right_scaled_penalty_deltas,
        }
    }
}

fn get_scaled_penalty_deltas_of_vpc_vector(
    extension: &Extension,
    vpc_vector: &Vec<Vpc>,
    cutoff: &Cutoff,
    penalties: &Penalty,
    traversed_anchors: &Vec<TraversedAnchor>,
) -> Vec<i64> {
    let scaled_penalty_delta_of_extension = (extension.length * cutoff.maximum_penalty_per_scale) as i64 - (extension.penalty * PREC_SCALE) as i64;

    let mut vpc_index_for_traversed_anchor = 0;
    let mut scaled_penalty_deltas: Vec<i64> = traversed_anchors.iter().rev().map(|traversed_anchor| {
        let length_to_traversed_start_position = extension.length - traversed_anchor.remained_length;
        let penalty_to_traversed_start_position = extension.penalty - traversed_anchor.remained_penalty;
        let min_query_length = length_to_traversed_start_position - (penalty_to_traversed_start_position / penalties.e);
        while min_query_length > vpc_vector[vpc_index_for_traversed_anchor].query_length {
            vpc_index_for_traversed_anchor += 1;
        }
        let remained_scaled_penalty_delta = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PREC_SCALE) as i64;
        vpc_vector[vpc_index_for_traversed_anchor].scaled_penalty_delta + remained_scaled_penalty_delta - scaled_penalty_delta_of_extension
    }).collect();
    scaled_penalty_deltas.reverse();

    scaled_penalty_deltas
}
