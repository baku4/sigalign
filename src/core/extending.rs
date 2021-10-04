use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Estimation, Extension, OwnedOperations, OperationsOfExtension, RefToOperations, StartPointOfOperations, CheckPoints};

mod dwfa;

use dwfa::DropoffWaveFront;

use std::collections::HashMap;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
    ) {
        for anchor in &mut self.anchors {
            // anchor.extend_right();
        }
    }
}

impl Anchor {
    fn extend_right(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        if !self.dropped {
            let record_slice = &record_sequence[self.record_position + self.size..];
            let query_slice = &query[self.query_position + self.size..];

            let record_slice_length = record_slice.len();
            let query_slice_length = query_slice.len();

            let spare_penalty = self.spare_penalty_of_right(penalties, cutoff, query_slice_length, record_slice_length);
        }
    }
    fn spare_penalty_of_right(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        let penalty_opposite_side = self.left_estimation.penalty;
        
        if penalty_opposite_side <= penalties.o {
            penalties.o
        } else {
            let length_opposite_side = self.left_estimation.length;
            self.spare_penalty(penalty_opposite_side, length_opposite_side, penalties, cutoff, query_slice_length, record_slice_length)
        }
    }
    fn spare_penalty(&self, penalty_opposite_side:usize, length_opposite_side: usize, penalties: &Penalties, cutoff: &Cutoff, query_length_this_side: usize, record_length_this_side: usize) -> usize {
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
    }
}
