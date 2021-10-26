use super::{PRECISION_SCALE, Cutoff, OwnedOperations};
use super::{AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType, AlignmentHashSet};
use super::{Anchors, Anchor, OperationsOfExtension, StartPointOfOperations};

use std::collections::HashSet;

type AnchorSymbol = Vec<usize>;

impl Anchors {
    pub fn get_alignment_results_for_semi_global(
        self,
        cutoff: &Cutoff,
    ) -> Vec<AlignmentResult> {
        let unique_anchors = self.get_unique_anchors(cutoff);

        let mut alignment_hash_set = AlignmentHashSet::new();

        unique_anchors.into_iter().filter_map(|unique_anchor_index| {
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

        let left_extension = anchor.left_extension.as_ref().unwrap();
        let right_extension = anchor.right_extension.as_ref().unwrap();
        
        let penalty = left_extension.penalty + right_extension.penalty;
        let length = left_extension.length + anchor.size + right_extension.length;

        let alignment_position_of_record = (
            anchor.record_position + left_extension.deletion_count as usize - left_extension.length ,
            anchor.record_position + anchor.size + right_extension.length - right_extension.deletion_count  as usize,
        );
        let alignment_position_of_query = (
            anchor.query_position + left_extension.insertion_count as usize - left_extension.length ,
            anchor.query_position + anchor.size + right_extension.length - right_extension.insertion_count  as usize,
        );
        let alignment_position = AlignmentPosition {
            record: alignment_position_of_record,
            query: alignment_position_of_query,
        };

        let alignment_is_new = alignment_hash_set.insert_and_check_new(penalty, alignment_position.clone());

        if alignment_is_new {
            let left_operations = match &left_extension.operations {
                OperationsOfExtension::Own(owned_operations) => owned_operations.operations.clone(),
                OperationsOfExtension::Ref(ref_to_operations) => {
                    let original_operation = match &self.anchors[ref_to_operations.anchor_index].left_extension.as_ref().unwrap().operations {
                        OperationsOfExtension::Own(owned_operations) => owned_operations,
                        _ => panic!("") // TODO: Write err msg.
                    };
                    original_operation.get_alignment_operations_from_start_point(&ref_to_operations.start_point_of_operations)
                },
            };
    
            let right_operations = match &right_extension.operations {
                OperationsOfExtension::Own(owned_operations) => owned_operations.operations.clone(),
                OperationsOfExtension::Ref(ref_to_operations) => {
                    let original_operation = match &self.anchors[ref_to_operations.anchor_index].right_extension.as_ref().unwrap().operations {
                        OperationsOfExtension::Own(owned_operations) => owned_operations,
                        _ => panic!("") // TODO: Write err msg.
                    };
                    original_operation.get_alignment_operations_from_start_point(&ref_to_operations.start_point_of_operations)
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
    fn get_unique_anchors(
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
                let left_extension = anchor.left_extension.as_ref().unwrap();
                let right_extension = anchor.right_extension.as_ref().unwrap();
                let penalty = left_extension.penalty + right_extension.penalty;
                let length = left_extension.length + anchor.size + right_extension.length;
                
                length >= cutoff.minimum_aligned_length
                && (PRECISION_SCALE * penalty / length) <= cutoff.penalty_per_scale
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

impl OwnedOperations {
    fn get_alignment_operations_from_start_point(&self, start_point: &StartPointOfOperations) -> Vec<AlignmentOperation> {
        let mut alignment_operations = self.operations[..=start_point.operation_index].to_vec();
        alignment_operations.last_mut().unwrap().count = start_point.operation_count;
        alignment_operations
    }
}