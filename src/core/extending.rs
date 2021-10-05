use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Estimation, Extension, OwnedOperations, OperationsOfExtension, RefToOperations, StartPointOfOperations, CheckPoints};

mod dwfa;

use dwfa::{DropoffWaveFront, PositionOfCheckpoint};

use std::collections::HashMap;

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
        println!("# extend left");
        self.extend_left_for_semi_global(record_sequence, query, penalties, cutoff);
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
                let extensions = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    let position_of_checkpoints = current_anchor.get_position_of_right_checkpoints(self);
                    current_anchor.get_right_extensions_for_semi_global(
                        current_anchor_index,
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                        position_of_checkpoints,
                    )
                };

                match extensions {
                    Some((extension_of_current_anchor, extension_of_checkpoints)) => {
                        self.anchors[current_anchor_index].right_extension = Some(extension_of_current_anchor);
                        for (checkpoint_anchor_index, extension_of_checkpoint) in extension_of_checkpoints {
                            self.anchors[current_anchor_index].connected_anchors.push(checkpoint_anchor_index);
                            self.anchors[checkpoint_anchor_index].right_extension = Some(extension_of_checkpoint);
                        }
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
                let extensions = {
                    let current_anchor = &self.anchors[current_anchor_index];
                    let position_of_checkpoints = current_anchor.get_position_of_left_checkpoints(self);
                    current_anchor.get_left_extensions_for_semi_global(
                        current_anchor_index,
                        record_sequence,
                        query,
                        penalties,
                        cutoff,
                        position_of_checkpoints,
                    )
                };

                match extensions {
                    Some((extension_of_current_anchor, extension_of_checkpoints)) => {
                        self.anchors[current_anchor_index].left_extension = Some(extension_of_current_anchor);
                        for (checkpoint_anchor_index, extension_of_checkpoint) in extension_of_checkpoints {
                            self.anchors[current_anchor_index].connected_anchors.push(checkpoint_anchor_index);
                            self.anchors[checkpoint_anchor_index].left_extension = Some(extension_of_checkpoint);
                        }
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
    fn get_right_extensions_for_semi_global(
        &self,
        current_anchor_index: usize,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        position_of_checkpoints: HashMap<usize, PositionOfCheckpoint>,
    ) -> Option<(Extension, HashMap<usize, Extension>)> {
        let record_slice = &record_sequence[self.record_position + self.size..];
        let query_slice = &query[self.query_position + self.size..];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_right(penalties, cutoff, query_slice_length, record_slice_length);

        #[cfg(test)]
        println!("# spare penalty right: {}", spare_penalty);

        DropoffWaveFront::align_right_for_semi_global(
            current_anchor_index,
            record_slice,
            query_slice,
            penalties,
            spare_penalty,
            position_of_checkpoints
        )
    }
    fn get_left_extensions_for_semi_global(
        &self,
        current_anchor_index: usize,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        position_of_checkpoints: HashMap<usize, PositionOfCheckpoint>,
    ) -> Option<(Extension, HashMap<usize, Extension>)> {
        let record_slice = &record_sequence[..self.record_position];
        let query_slice = &query[..self.query_position];

        let record_slice_length = record_slice.len();
        let query_slice_length = query_slice.len();

        let spare_penalty = self.spare_penalty_of_left(penalties, cutoff, query_slice_length, record_slice_length);

        #[cfg(test)]
        println!("# spare penalty left: {}", spare_penalty);

        DropoffWaveFront::align_left_for_semi_global(
            current_anchor_index,
            record_slice,
            query_slice,
            penalties,
            spare_penalty,
            position_of_checkpoints
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
    fn get_position_of_right_checkpoints(&self, anchors: &Anchors) -> HashMap<usize, PositionOfCheckpoint> {
        let mut position_of_checkpoints = HashMap::with_capacity(self.right_checkpoints.0.len());

        for &checkpoint_index in &self.right_checkpoints.0 {
            let checkpoint = &anchors.anchors[checkpoint_index];

            let ref_gap = checkpoint.record_position + checkpoint.size - self.record_position - self.size;
            let qry_gap = checkpoint.query_position + checkpoint.size - self.query_position - self.size;
            let position_of_checkpoint = PositionOfCheckpoint::new(ref_gap, qry_gap, checkpoint.size);

            position_of_checkpoints.insert(checkpoint_index, position_of_checkpoint);
        }
        position_of_checkpoints
    }
    fn get_position_of_left_checkpoints(&self, anchors: &Anchors) -> HashMap<usize, PositionOfCheckpoint> {
        let mut position_of_checkpoints = HashMap::with_capacity(self.left_checkpoints.0.len());

        for &checkpoint_index in &self.left_checkpoints.0 {
            let checkpoint = &anchors.anchors[checkpoint_index];

            let ref_gap = self.record_position - checkpoint.record_position;
            let qry_gap = self.query_position - checkpoint.query_position;
            let position_of_checkpoint = PositionOfCheckpoint::new(ref_gap, qry_gap, checkpoint.size);

            position_of_checkpoints.insert(checkpoint_index, position_of_checkpoint);
        }
        position_of_checkpoints
    }
}
