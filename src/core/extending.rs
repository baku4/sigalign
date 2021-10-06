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
        for current_anchor_index in 0..self.anchors.len() { // Extend from left to right
            if self.anchors[current_anchor_index].need_right_extension() {
                let extension = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    current_anchor.get_right_extension_for_semi_global(
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                    )
                };

                match extension {
                    Some(owned_extension) => {
                        // TODO:
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
        for current_anchor_index in (0..self.anchors.len()).rev() { // Extend from right to left
            if self.anchors[current_anchor_index].need_left_extension() {
                let extension = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    current_anchor.get_left_extension_for_semi_global(
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                    )
                };

                match extension {
                    Some(owned_extension) => {
                        // TODO:
                    },
                    None => {
                        self.anchors[current_anchor_index].dropped = true;
                    },
                }
            }
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

struct TraverseCandidates {
    previous_anchors: Vec<usize>,
    checkpoint: CheckPoint,
}