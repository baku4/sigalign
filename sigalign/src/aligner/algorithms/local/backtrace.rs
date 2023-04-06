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
use super::{Extension, WaveFront, calculate_spare_penalty};
use super::{Vpc, VpcIndexPackage};
use super::LocalExtension;
use ahash::AHashSet;

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