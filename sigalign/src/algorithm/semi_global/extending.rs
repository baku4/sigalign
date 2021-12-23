use super::{PRECISION_SCALE, Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, ReferableExtension, ExtensionReference, OperationReference, StartPointOfOperations, CheckPoints, CheckPoint};
use super::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};


// mod dwfa;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        self.extend_right_for_semi_global(record_sequence, query, penalties, cutoff, wave_front);
        self.extend_left_for_semi_global(record_sequence, query, penalties, cutoff, wave_front);
    }
    fn extend_right_for_semi_global(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        for current_anchor_index in 0..self.anchors.len() { // Extend from left to right
            if self.anchors[current_anchor_index].need_right_extension() {
                let current_anchor = &self.anchors[current_anchor_index];
                current_anchor.extend_wave_front_to_right(
                    record_sequence,
                    query,
                    penalties,
                    cutoff,
                    wave_front,
                );

                let optional_extension = wave_front.backtrace_from_the_end(penalties);

                match optional_extension {
                    Some(owned_extension) => {
                        self.right_traverse_check_from_owned_extension(
                            current_anchor_index,
                            owned_extension,
                            penalties
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
        wave_front: &mut WaveFront,
    ) {
        for current_anchor_index in (0..self.anchors.len()).rev() { // Extend from right to left
            if self.anchors[current_anchor_index].need_left_extension() {
                let current_anchor = &self.anchors[current_anchor_index];
                current_anchor.extend_wave_front_to_left(
                    record_sequence,
                    query,
                    penalties,
                    cutoff,
                    wave_front,
                );

                let optional_extension = wave_front.backtrace_from_the_end(penalties);

                match optional_extension {
                    Some(owned_extension) => {
                        self.left_traverse_check_from_owned_extension(
                            current_anchor_index,
                            owned_extension,
                            penalties
                        );
                    },
                    None => {
                        self.anchors[current_anchor_index].dropped = true;
                    },
                }
            }
        }
    }
    fn right_traverse_check_from_owned_extension( //FIXME: deduplicate code
        &mut self,
        original_anchor_index: usize,
        original_owned_extension: Extension,
        penalties: &Penalties,
    ) {
        let mut traverse_candidates = {
            let original_anchor = &mut self.anchors[original_anchor_index];

            original_anchor.right_referable_extension = Some(
                ReferableExtension::Own(original_owned_extension.clone())
            );
            
            let checkpoints = &original_anchor.right_checkpoints;
            checkpoints.to_first_traverse_candidates(original_anchor_index)
        };
        
        let original_penalty = original_owned_extension.penalty;
        let original_length = original_owned_extension.length;
        let original_insertion_count = original_owned_extension.insertion_count;
        let original_deletion_count = original_owned_extension.deletion_count;
        let original_operations = original_owned_extension.operations;

        let mut accumulated_penalty = 0;
        let mut accumulated_length = 0;
        let mut insertion_count = 0;
        let mut deletion_count = 0;

        for (operation_index, AlignmentOperation { alignment_type, count }) in original_operations.into_iter().enumerate().rev() {
            match alignment_type {
                AlignmentType::Match => {
                    let mut to_delete_traverse_candidates = Vec::new();

                    for traverse_candidate_index in 0..traverse_candidates.len() {
                        match traverse_candidates[traverse_candidate_index].checkpoint.is_traversed(
                            accumulated_length - deletion_count,
                            accumulated_length - insertion_count,
                            count
                        ) {
                            TraverseMarker::Traversed(count_of_checkpoint) => {
                                let traversed_anchor_index = traverse_candidates[traverse_candidate_index].checkpoint.anchor_index;
                                let checkpoint_anchor = &mut self.anchors[traversed_anchor_index];

                                let checkpoints_of_checkpoint_anchor = &checkpoint_anchor.right_checkpoints;

                                // Add extension to checkpoint
                                match checkpoint_anchor.right_referable_extension {
                                    None => {
                                        let operation_reference = OperationReference {
                                            anchor_index: original_anchor_index,
                                            start_point_of_operations: StartPointOfOperations {
                                                operation_index: operation_index, 
                                                operation_count: count_of_checkpoint,
                                            }
                                        };
                                        let extension_reference = ExtensionReference {
                                            penalty: original_penalty - accumulated_penalty,
                                            length: (original_length as u32 - accumulated_length - count + count_of_checkpoint) as usize,
                                            insertion_count: original_insertion_count - insertion_count,
                                            deletion_count: original_deletion_count - deletion_count,
                                            operation_reference: operation_reference,
                                        };

                                        checkpoint_anchor.right_referable_extension = Some(
                                            ReferableExtension::Ref(extension_reference)
                                        );
                                    },
                                    _ => {},
                                }

                                // Add new traverse candidate to candidates
                                for checkpoint_of_checkpoint_anchor in &checkpoints_of_checkpoint_anchor.0 {
                                    let new_traverse_candidate = traverse_candidates[traverse_candidate_index].to_new_candidate(checkpoint_of_checkpoint_anchor);
                                    traverse_candidates.push(new_traverse_candidate);
                                }

                                // Add this checkpoint to connected list of all previous anchor
                                for &previous_anchor_index in &traverse_candidates[traverse_candidate_index].previous_anchors {
                                    self.anchors[previous_anchor_index].connected_anchors.push(traversed_anchor_index);
                                }

                                // Add this traverse candidate to delete list
                                to_delete_traverse_candidates.push(traverse_candidate_index);
                            },
                            TraverseMarker::NotYetTraversed => {
                                // nothing to do
                            },
                            TraverseMarker::Passed => {
                                to_delete_traverse_candidates.push(traverse_candidate_index);
                            },
                        }
                    }
                    
                    for to_delete_traverse_candidate_index in to_delete_traverse_candidates.into_iter().rev() {
                        traverse_candidates.remove(to_delete_traverse_candidate_index);
                    }

                    if traverse_candidates.is_empty() {
                        return;
                    }

                    accumulated_length += count;
                },
                AlignmentType::Subst => {
                    accumulated_penalty += penalties.x * count as usize;
                    accumulated_length += count;
                },
                AlignmentType::Insertion => {
                    accumulated_penalty += penalties.o;
                    accumulated_penalty += penalties.e * count as usize;
                    accumulated_length += count;
                    insertion_count += count;
                },
                AlignmentType::Deletion => {
                    accumulated_penalty += penalties.o;
                    accumulated_penalty += penalties.e * count as usize;
                    accumulated_length += count;
                    deletion_count += count;
                },
            }
        }
    }
    fn left_traverse_check_from_owned_extension( //FIXME: deduplicate code
        &mut self,
        original_anchor_index: usize,
        original_owned_extension: Extension,
        penalties: &Penalties,
    ) {
        let mut traverse_candidates = {
            let original_anchor = &mut self.anchors[original_anchor_index];

            original_anchor.left_referable_extension = Some(
                ReferableExtension::Own(original_owned_extension.clone())
            );
            
            let checkpoints = &original_anchor.left_checkpoints;
            checkpoints.to_first_traverse_candidates(original_anchor_index)
        };
        
        let original_penalty = original_owned_extension.penalty;
        let original_length = original_owned_extension.length;
        let original_insertion_count = original_owned_extension.insertion_count;
        let original_deletion_count = original_owned_extension.deletion_count;
        let original_operations = original_owned_extension.operations;

        let mut accumulated_penalty = 0;
        let mut accumulated_length = 0;
        let mut insertion_count = 0;
        let mut deletion_count = 0;

        for (operation_index, AlignmentOperation { alignment_type, count }) in original_operations.into_iter().enumerate().rev() {
            match alignment_type {
                AlignmentType::Match => {
                    let mut to_delete_traverse_candidates = Vec::new();

                    for traverse_candidate_index in 0..traverse_candidates.len() {
                        match traverse_candidates[traverse_candidate_index].checkpoint.is_traversed(
                            accumulated_length - deletion_count,
                            accumulated_length - insertion_count,
                            count
                        ) {
                            TraverseMarker::Traversed(count_of_checkpoint) => {
                                let traversed_anchor_index = traverse_candidates[traverse_candidate_index].checkpoint.anchor_index;
                                let checkpoint_anchor = &mut self.anchors[traversed_anchor_index];

                                let checkpoints_of_checkpoint_anchor = &checkpoint_anchor.left_checkpoints;

                                // Add extension to checkpoint
                                match checkpoint_anchor.left_referable_extension {
                                    None => {
                                        let operation_reference = OperationReference {
                                            anchor_index: original_anchor_index,
                                            start_point_of_operations: StartPointOfOperations {
                                                operation_index: operation_index, 
                                                operation_count: count_of_checkpoint,
                                            }
                                        };
                                        let extension_reference = ExtensionReference {
                                            penalty: original_penalty - accumulated_penalty,
                                            length: (original_length as u32 - accumulated_length - count + count_of_checkpoint) as usize,
                                            insertion_count: original_insertion_count - insertion_count,
                                            deletion_count: original_deletion_count - deletion_count,
                                            operation_reference: operation_reference,
                                        };

                                        checkpoint_anchor.left_referable_extension = Some(
                                            ReferableExtension::Ref(extension_reference)
                                        );
                                    },
                                    _ => {},
                                }

                                // Add new traverse candidate to candidates
                                for checkpoint_of_checkpoint_anchor in &checkpoints_of_checkpoint_anchor.0 {
                                    let new_traverse_candidate = traverse_candidates[traverse_candidate_index].to_new_candidate(checkpoint_of_checkpoint_anchor);
                                    traverse_candidates.push(new_traverse_candidate);
                                }

                                // Add this checkpoint to connected list of all previous anchor
                                for &previous_anchor_index in &traverse_candidates[traverse_candidate_index].previous_anchors {
                                    self.anchors[previous_anchor_index].connected_anchors.push(traversed_anchor_index);
                                }

                                // Add this traverse candidate to delete list
                                to_delete_traverse_candidates.push(traverse_candidate_index);
                            },
                            TraverseMarker::NotYetTraversed => {
                                // nothing to do
                            },
                            TraverseMarker::Passed => {
                                to_delete_traverse_candidates.push(traverse_candidate_index);
                            },
                        }
                    }
                    
                    for to_delete_traverse_candidate_index in to_delete_traverse_candidates.into_iter().rev() {
                        traverse_candidates.remove(to_delete_traverse_candidate_index);
                    }

                    if traverse_candidates.is_empty() {
                        return;
                    }

                    accumulated_length += count;
                },
                AlignmentType::Subst => {
                    accumulated_penalty += penalties.x * count as usize;
                    accumulated_length += count;
                },
                AlignmentType::Insertion => {
                    accumulated_penalty += penalties.o;
                    accumulated_penalty += penalties.e * count as usize;
                    accumulated_length += count;
                    insertion_count += count;
                },
                AlignmentType::Deletion => {
                    accumulated_penalty += penalties.o;
                    accumulated_penalty += penalties.e * count as usize;
                    accumulated_length += count;
                    deletion_count += count;
                },
            }
        }
    }
}

impl Anchor {
    fn extend_wave_front_to_right(
        &self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        let record_slice = &record_sequence[self.record_position + self.size..];
        let query_slice = &query[self.query_position + self.size..];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_right(penalties, cutoff, query_slice_length, record_slice_length);

        wave_front.align_right_to_end_point(record_slice, query_slice, penalties, spare_penalty);
    }
    fn extend_wave_front_to_left(
        &self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) {
        let record_slice = &record_sequence[..self.record_position];
        let query_slice = &query[..self.query_position];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_left(penalties, cutoff, query_slice_length, record_slice_length);

        wave_front.align_right_to_end_point(record_slice, query_slice, penalties, spare_penalty);
    }
    fn spare_penalty_of_right(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        let penalty_opposite_side = self.left_estimation.penalty;
        let length_opposite_side = self.left_estimation.length;

        self.spare_penalty(penalty_opposite_side, length_opposite_side, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty_of_left(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        let (penalty_opposite_side, length_opposite_side) = self.right_referable_extension.as_ref().unwrap().penalty_and_length();

        self.spare_penalty(penalty_opposite_side, length_opposite_side, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty(
        &self,
        penalty_opposite_side:usize,
        length_opposite_side: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        query_length_this_side: usize,
        record_length_this_side: usize,
    ) -> usize {
        i64::max(
            penalties.o as i64,
            (
                penalties.e as i64 * (
                    (cutoff.maximum_penalty_per_scale * length_opposite_side) as i64
                    - (penalties.e * PRECISION_SCALE * penalty_opposite_side) as i64
                )
                + cutoff.maximum_penalty_per_scale as i64 * (
                    (
                        penalties.e  * (
                            self.size + query_length_this_side.min(record_length_this_side)
                        )
                    ) as i64 - penalties.o as i64
                )
            ) / (
                PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
            ) as i64 + 1
        ) as usize
    }
    fn need_right_extension(&self) -> bool {
        !self.dropped && match self.right_referable_extension {
            None => true,
            _ => false,
        }
    }
    fn need_left_extension(&self) -> bool {
        !self.dropped && match self.left_referable_extension {
            None => true,
            _ => false,
        }
    }
    // TODO: Can be deleted?
    // fn add_owned_extension_and_get_right_traverse_candidates(&mut self, original_anchor_index: usize, original_owned_extension: &Extension) -> Vec<TraverseCandidate> {
    //     self.right_extension = Some(original_owned_extension.clone());
    //     self.right_checkpoints.to_first_traverse_candidates(original_anchor_index)
    // }
}

impl CheckPoints {
    fn to_first_traverse_candidates(&self, original_anchor_index: usize) -> Vec<TraverseCandidate> {
        let previous_anchors = vec![original_anchor_index];
        self.0.iter().map(|checkpoint| {
            TraverseCandidate {
                previous_anchors: previous_anchors.clone(),
                checkpoint: checkpoint.clone(),
            }
        }).collect()
    }
}

impl CheckPoint {
    fn is_traversed(
        &self,
        record_position_of_match_start: u32,
        query_position_of_match_start: u32,
        count: u32,
    ) -> TraverseMarker {
        if let Some(record_gap_from_operation) = self.record_position_gap.checked_sub(record_position_of_match_start) {
            if let Some(query_gap_from_operation) = self.query_position_gap.checked_sub(query_position_of_match_start) {
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
    fn from_previous_checkpoint(&self, previous_checkpoint: &Self) -> Self {
        Self {
            anchor_index: self.anchor_index,
            anchor_size: self.anchor_size,
            record_position_gap: self.record_position_gap + previous_checkpoint.record_position_gap,
            query_position_gap: self.query_position_gap + previous_checkpoint.query_position_gap,
        }
    }
}

enum TraverseMarker {
    Traversed(u32),
    NotYetTraversed,
    Passed,
}

#[derive(Debug)]
struct TraverseCandidate {
    previous_anchors: Vec<usize>,
    checkpoint: CheckPoint,
}
impl TraverseCandidate {
    fn to_new_candidate(&self, new_checkpoint: &CheckPoint) -> Self {
        let mut new_previous_anchors = self.previous_anchors.clone();
        new_previous_anchors.push(self.checkpoint.anchor_index);
        Self {
            previous_anchors: new_previous_anchors,
            checkpoint: new_checkpoint.from_previous_checkpoint(&self.checkpoint)
        }
    }
}