use super::{PRECISION_SCALE, Cutoff, OwnedOperations};
use super::{AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType, AlignmentHashSet};
use super::{Anchors, Anchor, ReferableExtension, ExtensionReference, OperationReference, StartPointOfOperations, CheckPoints, CheckPoint};
use super::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

use std::collections::HashSet;

type AnchorSymbol = Vec<usize>;

impl Anchors {
    pub fn get_alignment_results_for_semi_global(
        self,
        cutoff: &Cutoff,
    ) -> Vec<AlignmentResult> {
        let valid_unique_anchors = self.get_valid_unique_anchors(cutoff);

        let mut alignment_hash_set = AlignmentHashSet::new();

        valid_unique_anchors.into_iter().filter_map(|unique_anchor_index| {
            self.get_optional_alignment_result_of_anchor_for_semi_global(
                unique_anchor_index,
                &mut alignment_hash_set,
            )
        }).collect()
    }
    fn get_optional_alignment_result_of_anchor_for_semi_global(
        &self,
        anchor_index: usize,
        alignment_hash_set: &mut AlignmentHashSet,
    ) -> Option<AlignmentResult> {
        let anchor = &self.anchors[anchor_index];

        let left_referable_extension = anchor.left_referable_extension.as_ref().unwrap();
        let right_referable_extension = anchor.right_referable_extension.as_ref().unwrap();

        let (left_penalty, left_length) = left_referable_extension.penalty_and_length();
        let (right_penalty, right_length) = right_referable_extension.penalty_and_length();
        
        let penalty = left_penalty + right_penalty;
        let length = left_length + anchor.size + right_length;

        let (left_insertion_count, left_deletion_count) = left_referable_extension.insertion_and_deletion_count();
        let (right_insertion_count, right_deletion_count) = right_referable_extension.insertion_and_deletion_count();

        let alignment_position_of_record = (
            anchor.record_position + left_deletion_count as usize - left_length,
            anchor.record_position + anchor.size + right_length - right_deletion_count  as usize,
        );
        let alignment_position_of_query = (
            anchor.query_position + left_insertion_count as usize - left_length,
            anchor.query_position + anchor.size + right_length - right_insertion_count  as usize,
        );
        let alignment_position = AlignmentPosition {
            record: alignment_position_of_record,
            query: alignment_position_of_query,
        };

        let alignment_is_new = alignment_hash_set.insert_and_check_new(penalty, alignment_position.clone());

        if alignment_is_new {
            let left_operations = match left_referable_extension {
                ReferableExtension::Own(extension) => {
                    extension.operations
                },
                ReferableExtension::Ref(extension_reference) => {
                    let operation_reference = &extension_reference.operation_reference;
                    match &self.anchors[operation_reference.anchor_index].left_referable_extension.as_ref().unwrap() {
                        ReferableExtension::Own(extension) => {
                            extension.get_alignment_operations_from_start_point(&operation_reference.start_point_of_operations)
                        },
                        _ => panic!("") // TODO: Write err msg.
                    }
                },
            };

            let right_operations = match right_referable_extension {
                ReferableExtension::Own(extension) => {
                    extension.operations
                },
                ReferableExtension::Ref(extension_reference) => {
                    let operation_reference = &extension_reference.operation_reference;
                    match &self.anchors[operation_reference.anchor_index].right_referable_extension.as_ref().unwrap() {
                        ReferableExtension::Own(extension) => {
                            extension.get_alignment_operations_from_start_point(&operation_reference.start_point_of_operations)
                        },
                        _ => panic!("") // TODO: Write err msg.
                    }
                },
            };
    
            let alignment_operations = AlignmentOperation::concatenate_operations(left_operations, right_operations, anchor.size as u32);
    
            Some(
                AlignmentResult {
                    dissimilarity: penalty as f32 / length as f32,
                    penalty,
                    length,
                    position: alignment_position,
                    operations: alignment_operations,
                }
            )
        } else {
            None
        }
    }
    fn get_valid_unique_anchors(
        &self,
        cutoff: &Cutoff,
    ) -> Vec<usize> {
        let valid_anchors = self.set_of_valid_anchors(cutoff);

        let mut used_symbols: HashSet<AnchorSymbol> = HashSet::with_capacity(valid_anchors.len());
        let mut unique_anchors: Vec<usize> = Vec::with_capacity(valid_anchors.len());

        for valid_anchor_index in valid_anchors {
            let symbol = self.anchors[valid_anchor_index].get_symbol(valid_anchor_index);
            let symbol_is_new = used_symbols.insert(symbol);
            if symbol_is_new {
                unique_anchors.push(valid_anchor_index);
            }
        };

        unique_anchors
    }
    fn set_of_valid_anchors(
        &self,
        cutoff: &Cutoff,
    ) -> HashSet<usize> {
        self.anchors.iter().enumerate().filter_map(|(anchor_index, anchor)| {
            if !anchor.dropped && {
                let left_referable_extension = anchor.left_referable_extension.as_ref().unwrap();
                let right_referable_extension = anchor.right_referable_extension.as_ref().unwrap();

                let (left_penalty, left_length) = left_referable_extension.penalty_and_length();
                let (right_penalty, right_length) = right_referable_extension.penalty_and_length();
                
                let penalty = left_penalty + right_penalty;
                let length = left_length + anchor.size + right_length;
                
                length >= cutoff.minimum_aligned_length
                && (PRECISION_SCALE * penalty / length) <= cutoff.maximum_penalty_per_scale
            } {
                Some(anchor_index)
            } else {
                None
            }
        }).collect()
    }
}

impl Anchor {
    fn get_symbol(&self, anchor_index: usize) -> AnchorSymbol {
        let mut symbol = self.connected_anchors.clone();
        symbol.push(anchor_index);
        symbol.sort();
        symbol
    }
}

impl Extension {
    fn get_alignment_operations_from_start_point(&self, start_point: &StartPointOfOperations) -> Vec<AlignmentOperation> {
        let mut alignment_operations = self.operations[..=start_point.operation_index].to_vec();
        alignment_operations.last_mut().unwrap().count = start_point.operation_count;
        alignment_operations
    }
}