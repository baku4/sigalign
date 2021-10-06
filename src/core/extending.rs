use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Extension, OperationsOfExtension, OwnedOperations, RefToOperations, StartPointOfOperations, CheckPoints, CheckPoint};

mod dwfa;

use dwfa::DropoffWaveFront;

use std::collections::{HashSet, HashMap};

impl Anchors {
    pub fn extend_for_semi_global(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        #[cfg(test)]
        println!("# extend right");
        self.extend_right_for_semi_global(record_sequence, query, penalties, cutoff);
        #[cfg(test)]
        println!("{:#?}", self);
        #[cfg(test)]
        println!("# extend left");
        self.extend_left_for_semi_global(record_sequence, query, penalties, cutoff);
        #[cfg(test)]
        println!("{:#?}", self);
    }
    fn extend_right_for_semi_global(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        for current_anchor_index in 0..self.anchors.len() {
            if self.anchors[current_anchor_index].need_right_extension() {
                let owned_extension = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    current_anchor.get_right_extension_for_semi_global(
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                    )
                };

                match owned_extension {
                    Some(owned_extension) => {
                        let owned_operations = match &owned_extension.operations {
                            OperationsOfExtension::Own(owned_operations) => owned_operations.clone(),
                            _ => panic!("") // TODO: Write err msg
                        };

                        self.add_right_extension_and_propagate_to_traversed_checkpoints(
                            penalties,
                            current_anchor_index,
                            (current_anchor_index, &owned_operations),
                            Vec::new(),
                            owned_extension,
                        );
                    },
                    None => {
                        self.anchors[current_anchor_index].dropped = true;
                    },
                }
            }
        }
    }
    fn extend_left_for_semi_global(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        for current_anchor_index in (0..self.anchors.len()).rev() {
            if self.anchors[current_anchor_index].need_left_extension() {
                let owned_extension = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    current_anchor.get_left_extension_for_semi_global(
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                    )
                };

                match owned_extension {
                    Some(owned_extension) => {
                        let owned_operations = match &owned_extension.operations {
                            OperationsOfExtension::Own(owned_operations) => owned_operations.clone(),
                            _ => panic!("") // TODO: Write err msg
                        };

                        self.add_left_extension_and_propagate_to_traversed_checkpoints(
                            penalties,
                            current_anchor_index,
                            (current_anchor_index, &owned_operations),
                            Vec::new(),
                            owned_extension,
                        );
                    },
                    None => {
                        self.anchors[current_anchor_index].dropped = true;
                    },
                }
            }
        }
    }
    fn add_right_extension_and_propagate_to_traversed_checkpoints(
        &mut self,
        penalties: &Penalties,
        traversed_anchor_index: usize,
        original_anchor_index_and_operations: (usize, &OwnedOperations),
        mut previous_anchors: Vec<usize>,
        extension: Extension,
    ) {
        self.anchors[traversed_anchor_index].right_extension = Some(extension);
        
        for &anchor_index in &previous_anchors {
            self.anchors[anchor_index].connected_anchors.push(traversed_anchor_index);
        }

        previous_anchors.push(traversed_anchor_index);

        let next_traversed_checkpoints_and_extensions: Vec<(usize, Extension)> = self.anchors[traversed_anchor_index].get_right_traversed_checkpoints_and_extensions(penalties, original_anchor_index_and_operations);
        #[cfg(test)]
        println!("# right - next_traversed_checkpoints_and_extensions\n{:?}", next_traversed_checkpoints_and_extensions);

        if next_traversed_checkpoints_and_extensions.len() == 0 {
            return;
        }

        for (next_traversed_anchor_index, next_extension) in next_traversed_checkpoints_and_extensions {
            self.add_right_extension_and_propagate_to_traversed_checkpoints(
                penalties,
                next_traversed_anchor_index, 
                original_anchor_index_and_operations,
                previous_anchors.clone(),
                next_extension,
            );
        }
    }
    fn add_left_extension_and_propagate_to_traversed_checkpoints(
        &mut self,
        penalties: &Penalties,
        traversed_anchor_index: usize,
        original_anchor_index_and_operations: (usize, &OwnedOperations),
        mut previous_anchors: Vec<usize>,
        extension: Extension,
    ) {
        self.anchors[traversed_anchor_index].left_extension = Some(extension);
        
        for &anchor_index in &previous_anchors {
            self.anchors[anchor_index].connected_anchors.push(traversed_anchor_index);
        }

        previous_anchors.push(traversed_anchor_index);

        let next_traversed_checkpoints_and_extensions: Vec<(usize, Extension)> = self.anchors[traversed_anchor_index].get_left_traversed_checkpoints_and_extensions(penalties, original_anchor_index_and_operations);

        if next_traversed_checkpoints_and_extensions.len() == 0 {
            return;
        }

        for (next_traversed_anchor_index, next_extension) in next_traversed_checkpoints_and_extensions {
            self.add_left_extension_and_propagate_to_traversed_checkpoints(
                penalties,
                next_traversed_anchor_index, 
                original_anchor_index_and_operations,
                previous_anchors.clone(),
                next_extension,
            );
        }
    }
}

impl Anchor {
    fn get_right_extension_for_semi_global(
        &self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> Option<Extension> {
        let record_slice = &record_sequence[self.record_position + self.size..];
        let query_slice = &query[self.query_position + self.size..];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_right(penalties, cutoff, query_slice_length, record_slice_length);

        DropoffWaveFront::align_right_for_semi_global(
            record_slice,
            query_slice,
            penalties,
            spare_penalty,
        )
    }
    fn get_left_extension_for_semi_global(
        &self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> Option<Extension> {
        let record_slice = &record_sequence[..self.record_position];
        let query_slice = &query[..self.query_position];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_left(penalties, cutoff, query_slice_length, record_slice_length);

        DropoffWaveFront::align_left_for_semi_global(
            record_slice,
            query_slice,
            penalties,
            spare_penalty,
        )
    }
    fn spare_penalty_of_right(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        let penalty_opposite_side = self.left_estimation.penalty;
        let length_opposite_side = self.left_estimation.length;

        self.spare_penalty(penalty_opposite_side, length_opposite_side, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty_of_left(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        let penalty_opposite_side = self.right_extension.as_ref().unwrap().penalty;
        let length_opposite_side = self.right_extension.as_ref().unwrap().length;

        self.spare_penalty(penalty_opposite_side, length_opposite_side, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty(&self, penalty_opposite_side:usize, length_opposite_side: usize, penalties: &Penalties, cutoff: &Cutoff, query_length_this_side: usize, record_length_this_side: usize) -> usize {
        penalties.o.max(
            (
                (
                    cutoff.penalty_per_length * (
                        penalties.e * (
                            length_opposite_side + self.size + query_length_this_side.min(record_length_this_side)
                        ) - penalties.o
                    ) as f32 - (
                        penalty_opposite_side * penalties.e
                    ) as f32
                ) / (
                    penalties.e as f32 - cutoff.penalty_per_length
                )
            ).ceil() as usize
        )
    }
    fn need_right_extension(&self) -> bool {
        !self.dropped && match self.right_extension {
            None => true,
            _ => false,
        }
    }
    fn need_left_extension(&self) -> bool {
        !self.dropped && match self.left_extension {
            None => true,
            _ => false,
        }
    }
    fn get_right_traversed_checkpoints_and_extensions(
        &self,
        penalties: &Penalties,
        original_anchor_index_and_operations: (usize, &OwnedOperations),
    ) -> Vec<(usize, Extension)> {
        let extension = self.right_extension.as_ref().unwrap();
        let checkpoints = &self.right_checkpoints;

        if checkpoints.0.len() == 0 {
            return Vec::new();
        }

        let (original_anchor_index, original_owned_operations) = original_anchor_index_and_operations;

        #[cfg(test)]
        println!("# original_anchor_index: {:?}", original_anchor_index);
        println!("# original_owned_operations: {:?}", original_owned_operations);

        let start_point_of_operations = match &extension.operations {
            OperationsOfExtension::Own(owned_operations) => {
                let operation_index = owned_operations.operations.len() - 1;
                let operation_count = owned_operations.operations[operation_index].count;
                StartPointOfOperations {
                    operation_index,
                    operation_count,
                }
            },
            OperationsOfExtension::Ref(ref_to_operations) => {
                ref_to_operations.start_point_of_operations.clone()
            },
        };
        original_owned_operations.get_traversed_checkpoints_and_extensions_from_point(
            original_anchor_index,
            extension.penalty,
            extension.length,
            checkpoints,
            penalties,
            &start_point_of_operations,
        )
    }
    fn get_left_traversed_checkpoints_and_extensions(
        &self,
        penalties: &Penalties,
        original_anchor_index_and_operations: (usize, &OwnedOperations),
    ) -> Vec<(usize, Extension)> {
        let extension = self.left_extension.as_ref().unwrap();
        let checkpoints = &self.left_checkpoints;

        if checkpoints.0.len() == 0 {
            return Vec::new();
        }

        let (original_anchor_index, original_owned_operations) = original_anchor_index_and_operations;

        let start_point_of_operations = match &extension.operations {
            OperationsOfExtension::Own(owned_operations) => {
                let operation_index = owned_operations.operations.len() - 1;
                let operation_count = owned_operations.operations[operation_index].count;
                StartPointOfOperations {
                    operation_index,
                    operation_count,
                }
            },
            OperationsOfExtension::Ref(ref_to_operations) => {
                ref_to_operations.start_point_of_operations.clone()
            },
        };
        original_owned_operations.get_traversed_checkpoints_and_extensions_from_point(
            original_anchor_index,
            extension.penalty,
            extension.length,
            checkpoints,
            penalties,
            &start_point_of_operations,
        )
    }
}

impl OwnedOperations {
    fn get_traversed_checkpoints_and_extensions_from_point(
        &self,
        original_anchor_index: usize,
        penalty_of_extension: usize,
        length_of_extension: usize,
        checkpoints: &CheckPoints,
        penalties: &Penalties,
        start_point_of_operations: &StartPointOfOperations,
    ) -> Vec<(usize, Extension)> {
        let checkpoint_count = checkpoints.0.len();
        let mut checkpoint_indices_to_traverse_check: HashSet<usize> = (0..checkpoint_count).collect();

        let mut traversed_checkpoints_and_extensions: Vec<(usize, Extension)> = Vec::with_capacity(checkpoint_count);

        let mut penalty = 0;
        let mut length = 0;
        let mut record_position_of_operation = 0;
        let mut query_position_of_operation = 0;

        // * Rationality
        // (1) First operation is always type of Match.
        // (2) First match operation can not traverse checkpoint. Because ungapped anchors are merged in 'anchoring' step.
        let first_count = start_point_of_operations.operation_count;
        length += first_count;
        record_position_of_operation += first_count;
        query_position_of_operation += first_count;

        if start_point_of_operations.operation_index != 0 {
            for (operation_index, AlignmentOperation { alignment_type, count }) in self.operations[..start_point_of_operations.operation_index].iter().enumerate().rev() {
                match alignment_type {
                    AlignmentType::Match => {
                        let record_position_of_match_start = record_position_of_operation;
                        let query_position_of_match_start = query_position_of_operation;
    
                        if checkpoint_indices_to_traverse_check.len() == 0 {
                            return traversed_checkpoints_and_extensions;
                        }
                        for checkpoint_index in checkpoint_indices_to_traverse_check.clone() {
                            let checkpoint = &checkpoints.0[checkpoint_index];
                            match checkpoint.is_traversed(record_position_of_match_start, query_position_of_match_start, *count) {
                                TraverseMarker::Traversed(count_of_reference) => {
                                    let extension = Extension {
                                        penalty: penalty_of_extension - penalty,
                                        length: length_of_extension - length as usize,
                                        operations: OperationsOfExtension::Ref(
                                            RefToOperations {
                                                anchor_index: original_anchor_index,
                                                start_point_of_operations: StartPointOfOperations {
                                                    operation_index,
                                                    operation_count: count_of_reference,
                                                }
                                            }
                                        )
                                    };

                                },
                                TraverseMarker::NotYetTraversed => {
                                    // nothing to do
                                },
                                TraverseMarker::Passed => {
                                    checkpoint_indices_to_traverse_check.remove(&checkpoint_index);
                                }
                            }
                        }
                        
                        length += count;
                        record_position_of_operation += count;
                        query_position_of_operation += count;
                    },
                    AlignmentType::Subst => {
                        penalty += penalties.x * *count as usize;
                        length += count;
                        record_position_of_operation += count;
                        query_position_of_operation += count;
                    },
                    AlignmentType::Insertion => {
                        penalty += penalties.o;
                        penalty += penalties.e * *count as usize;
                        length += count;
                        record_position_of_operation += count;
                    },
                    AlignmentType::Deletion => {
                        penalty += penalties.o;
                        penalty += penalties.e * *count as usize;
                        length += count;
                        query_position_of_operation += count;
                    },
                }
            }
        }

        traversed_checkpoints_and_extensions
    }
}

impl CheckPoint {
    fn is_traversed(
        &self,
        record_position_of_match_start: u32,
        query_position_of_match_start: u32,
        count: u32,
    ) -> TraverseMarker {
        if let Some(record_gap_from_operation) = record_position_of_match_start.checked_sub(self.record_position_gap) {
            if let Some(query_gap_from_operation) = query_position_of_match_start.checked_sub(self.query_position_gap) {
                if record_gap_from_operation == query_gap_from_operation {
                    if let Some(count_of_checkpoint) = count.checked_sub(record_gap_from_operation + self.anchor_size) {
                        return TraverseMarker::Traversed(count_of_checkpoint);
                    }
                }
                return TraverseMarker::NotYetTraversed;
            }
        }
        return TraverseMarker::Passed;
    }
}

enum TraverseMarker {
    Traversed(u32),
    NotYetTraversed,
    Passed,
}
