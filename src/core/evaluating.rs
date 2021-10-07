use super::{Cutoff, OwnedOperations};
use super::{AlignmentResultOfRecord, AlignmentPosition, AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Extension, OperationsOfExtension, RefToOperations, StartPointOfOperations};

use std::collections::HashSet;

type AnchorSymbol = Vec<usize>;

impl Anchors {
    pub fn get_alignment_result_for_semi_global(
        self,
        cutoff: &Cutoff,
    ) -> Vec<AlignmentResultOfRecord> {
        let unique_anchors = self.get_unique_anchors(cutoff);

        unique_anchors.into_iter().map(|unique_anchor_index| {
            self.get_alignment_result_of_anchor_for_semi_global(unique_anchor_index)
        }).collect()
    }
    fn get_alignment_result_of_anchor_for_semi_global(
        &self,
        anchor_index: usize,
    ) -> AlignmentResultOfRecord {
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

        let mut left_operations = match &left_extension.operations {
            OperationsOfExtension::Own(owned_operations) => owned_operations.operations.clone(),
            OperationsOfExtension::Ref(ref_to_operations) => {
                let original_operation = match &self.anchors[ref_to_operations.anchor_index].left_extension.as_ref().unwrap().operations {
                    OperationsOfExtension::Own(owned_operations) => owned_operations,
                    _ => panic!("") // TODO: Write err msg.
                };
                original_operation.get_alignment_operations_from_start_point(&ref_to_operations.start_point_of_operations)
            },
        };

        let mut right_operations = match &right_extension.operations {
            OperationsOfExtension::Own(owned_operations) => owned_operations.operations.clone(),
            OperationsOfExtension::Ref(ref_to_operations) => {
                let original_operation = match &self.anchors[ref_to_operations.anchor_index].right_extension.as_ref().unwrap().operations {
                    OperationsOfExtension::Own(owned_operations) => owned_operations,
                    _ => panic!("") // TODO: Write err msg.
                };
                original_operation.get_alignment_operations_from_start_point(&ref_to_operations.start_point_of_operations)
            },
        };
        right_operations.reverse();

        // Add anchor sized Match operation to left operations
        if let AlignmentOperation {
            alignment_type: AlignmentType::Match,
            count,
        } = left_operations.last_mut().unwrap() {
            *count += anchor.size as u32;
        } else {
            left_operations.push(
                AlignmentOperation {
                    alignment_type: AlignmentType::Match,
                    count: anchor.size as u32,
                }
            );
        };

        // Add right operations to left operations
        if let AlignmentOperation {
            alignment_type: AlignmentType::Match,
            count: right_count,
        } = right_operations.first_mut().unwrap() {
            if let AlignmentOperation {
                alignment_type: AlignmentType::Match,
                count: left_count,
            } = left_operations.last_mut().unwrap() {
                *left_count += *right_count;
            }
            right_operations.remove(0);
        };

        left_operations.append(&mut right_operations);

        AlignmentResultOfRecord {
            dissimilarity: penalty as f32 / length as f32,
            penalty,
            length,
            position: alignment_position,
            operations: left_operations,
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
                
                length >= cutoff.minimum_aligned_length && (penalty as f32 / length as f32) <= cutoff.penalty_per_length
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