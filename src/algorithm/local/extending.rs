use super::{Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor, Extension};
use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};

mod dwfa;

use dwfa::PointOfMaximumLength;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        self.anchors.iter_mut().for_each(|anchor| {
            let optional_extensions = anchor.get_extensions_for_local(record_sequence, query, penalties, cutoff);

            match optional_extensions {
                Some((left_extension, right_extension)) => {
                    anchor.left_extension = Some(left_extension);
                    anchor.right_extension = Some(right_extension);
                },
                None => {
                    anchor.dropped = true;
                },
            }
        });
    }
}

impl Anchor {
    fn get_extensions_for_local(
        &self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) -> Option<(Extension, Extension)> {
        let right_record_slice = &record_sequence[self.record_position + self.size..];
        let right_query_slice = &query[self.query_position + self.size..];
        let right_record_slice_length = right_record_slice.len();
        let right_query_slice_length = right_query_slice.len();
        let right_spare_penalty = self.spare_penalty_of_right(penalties, cutoff, right_query_slice_length, right_record_slice_length);

        let right_dropoff_wave_front = DropoffWaveFront::aligned_right_for_local(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_point_of_maximum_length = right_dropoff_wave_front.point_of_maximum_length();

        let left_record_slice = &record_sequence[..self.record_position];
        let left_query_slice = &query[..self.query_position];
        let left_record_slice_length = left_record_slice.len();
        let left_query_slice_length = left_query_slice.len();
        let spare_penalty_padding_of_right = right_point_of_maximum_length.spare_penalty_padding(cutoff);
        let left_spare_penalty = self.spare_penalty_of_left(spare_penalty_padding_of_right, penalties, cutoff, left_query_slice_length, left_record_slice_length);

        let left_dropoff_wave_front = DropoffWaveFront::aligned_left_for_local(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_point_of_maximum_length = left_dropoff_wave_front.point_of_maximum_length();

        let point_of_maximum_length = PointOfMaximumLength::get_optional_start_point_of_wave_front(left_point_of_maximum_length, right_point_of_maximum_length, self.size, cutoff);

        match point_of_maximum_length {
            Some(start_point_of_wave_front) => {
                Some((
                    left_dropoff_wave_front.backtrace_from_start_point_of_wave_front(
                        start_point_of_wave_front.left_score,
                        start_point_of_wave_front.left_index_of_components,
                        penalties
                    ),
                    right_dropoff_wave_front.backtrace_from_start_point_of_wave_front(
                        start_point_of_wave_front.right_score,
                        start_point_of_wave_front.right_index_of_components,
                        penalties
                    ),
                ))
            },
            None => {
                None
            }
        }
    }
    fn spare_penalty_of_right(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        self.spare_penalty(self.spare_penalty_padding_of_left, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty_of_left(&self, spare_penalty_padding_of_right: f32, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        self.spare_penalty(spare_penalty_padding_of_right, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty(&self, spare_penalty_padding: f32, penalties: &Penalties, cutoff: &Cutoff, query_length_this_side: usize, record_length_this_side: usize) -> usize {
        penalties.o.max(
            (
                penalties.e as f32 * (spare_penalty_padding + (
                    self.size + query_length_this_side.min(record_length_this_side)
                ) as f32) - (
                    cutoff.penalty_per_length * penalties.o as f32
                ) / (
                    penalties.e as f32 - cutoff.penalty_per_length
                )
            ).ceil() as usize
        )
    }
}
