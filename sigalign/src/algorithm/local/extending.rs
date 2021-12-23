use super::{PRECISION_SCALE, Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentType};
use super::{Anchors, Anchor};
// TODO: Delete
// use super::{DropoffWaveFront, WaveFrontScore, Components, Component};
// use super::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};
use super::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

// TODO: Delete
// mod dwfa;

impl Anchors {
    pub fn extend(
        &mut self,
        record_sequence: Sequence,
        query: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) {
        self.anchors.iter_mut().for_each(|anchor| {
            let optional_extensions = anchor.get_extensions_for_local(record_sequence, query, penalties, cutoff, left_wave_front, right_wave_front);

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
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> Option<(Extension, Extension)> {
        let right_record_slice = &record_sequence[self.record_position + self.size..];
        let right_query_slice = &query[self.query_position + self.size..];
        let right_record_slice_length = right_record_slice.len();
        let right_query_slice_length = right_query_slice.len();
        let right_spare_penalty = self.spare_penalty_of_right(penalties, cutoff, right_query_slice_length, right_record_slice_length);

        right_wave_front.align_right_to_end_point(record_sequence, query, penalties, right_spare_penalty);
        let right_point_of_maximum_length = right_wave_front.point_of_maximum_length();

        let left_record_slice = &record_sequence[..self.record_position];
        let left_query_slice = &query[..self.query_position];
        let left_record_slice_length = left_record_slice.len();
        let left_query_slice_length = left_query_slice.len();
        let spare_penalty_determinant_of_right = right_point_of_maximum_length.spare_penalty_determinant(cutoff);
        let left_spare_penalty = self.spare_penalty_of_left(spare_penalty_determinant_of_right, penalties, cutoff, left_query_slice_length, left_record_slice_length);

        left_wave_front.align_left_to_end_point(record_sequence, query, penalties, left_spare_penalty);
        let left_point_of_maximum_length = left_wave_front.point_of_maximum_length();

        let point_of_maximum_length = PointOfMaximumLength::get_optional_start_points_of_wave_front(left_point_of_maximum_length, right_point_of_maximum_length, self.size, cutoff);

        match point_of_maximum_length {
            Some(start_point_of_wave_front) => {
                Some((
                    left_wave_front.backtrace_from_point(
                        start_point_of_wave_front.left_score,
                        start_point_of_wave_front.left_index_of_components,
                        penalties,
                    ),
                    right_wave_front.backtrace_from_point(
                        start_point_of_wave_front.right_score,
                        start_point_of_wave_front.right_index_of_components,
                        penalties,
                    ),
                ))
            },
            None => {
                None
            }
        }
    }
    fn spare_penalty_of_right(&self, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        self.spare_penalty(self.spare_penalty_determinant_of_left, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty_of_left(&self, spare_penalty_determinant_of_right: i64, penalties: &Penalties, cutoff: &Cutoff, query_slice_length: usize, record_slice_length: usize) -> usize {
        self.spare_penalty(spare_penalty_determinant_of_right, penalties, cutoff, query_slice_length, record_slice_length)
    }
    fn spare_penalty(
        &self,
        spare_penalty_determinant: i64,
        penalties: &Penalties,
        cutoff: &Cutoff,
        query_length_this_side: usize,
        record_length_this_side: usize,
    ) -> usize {
        i64::max(
            penalties.o as i64,
            (
                penalties.e as i64 * spare_penalty_determinant
                + cutoff.maximum_penalty_per_scale as i64 * (
                    (
                        penalties.e * (
                            self.size + query_length_this_side.min(record_length_this_side)
                        )
                    ) as i64 - penalties.o as i64
                )
            ) / (
                PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
            ) as i64 + 1
        ) as usize
    }
}

impl WaveFront {
    fn point_of_maximum_length(&self) -> PointOfMaximumLength {
        let wave_front_scores = &self.wave_front_scores[..=self.end_point.score];
        
        let index_of_components_and_maximum_length_of_scores = wave_front_scores.iter().map(|wave_front_score| {
            wave_front_score.index_and_maximum_length()
        }).enumerate().collect();
        PointOfMaximumLength {
            index_of_components_and_maximum_length_of_scores,
        }
    }
}

impl WaveFrontScore {
    fn index_and_maximum_length(&self) -> (usize, i32) {
        let optional_index_and_maximum_length = self.components_by_k.iter().enumerate()
            .filter_map(|(component_index, components)| {
                match components.m.optional_length() {
                    Some(length) => Some((component_index, length)),
                    None => None,
                }
            }).max_by_key(|(_, length)| *length);

        match optional_index_and_maximum_length {
            Some(index_and_maximum_length) => {
                index_and_maximum_length
            },
            None => {
                (0, 0)
            },
        }
    }
}

impl Component {
    fn optional_length(&self) -> Option<i32> {
        match self.bt {
            BackTraceMarker::Empty => None,
            _ => Some(self.fr + self.deletion_count as i32)
        }
    }
}

#[derive(Debug)]
pub struct PointOfMaximumLength {
    // (score, (index, length))
    index_of_components_and_maximum_length_of_scores: Vec<(usize, (usize, i32))>,
}

impl PointOfMaximumLength {
    pub fn spare_penalty_determinant(
        &self,
        cutoff: &Cutoff,
    ) -> i64 {
        // Spare penalty determinant:
        // penalty per scale * length - PRECISION_SCALE * penalty
        let mut maximum_determinant: i64 = i64::MIN;

        let penalty_per_scale = cutoff.maximum_penalty_per_scale as i64;
        self.index_of_components_and_maximum_length_of_scores.iter().for_each(|(score, (_, length))| {
            let determinant = penalty_per_scale * *length as i64 - (PRECISION_SCALE * *score) as i64;
            if maximum_determinant < determinant {
                maximum_determinant = determinant;
            }
        });

        maximum_determinant
    }
    pub fn get_optional_start_points_of_wave_front(
        left: Self,
        right: Self,
        anchor_size: usize,
        cutoff: &Cutoff,
    ) -> Option<StartPointOfWaveFront> {
        let mut left_sorted_point = left.index_of_components_and_maximum_length_of_scores;
        left_sorted_point.sort_unstable_by_key(|(_, (_, length))| *length);
        let mut right_sorted_point = right.index_of_components_and_maximum_length_of_scores;
        right_sorted_point.sort_unstable_by_key(|(_, (_, length))| *length);

        let mut optional_start_point_of_wave_front: Option<StartPointOfWaveFront> = None;
        let mut length_of_start_point = 0;
        let mut penalty_per_scale_of_start_point = usize::MAX;

        let mut right_start_index = 0;

        let left_sorted_point_count = left_sorted_point.len();
        let right_sorted_point_count = right_sorted_point.len();

        'left_loop: for left_index in (0..left_sorted_point_count).rev() {
            'right_loop: for right_index in (right_start_index..right_sorted_point_count).rev() {
                let &(left_penalty, (left_index_of_components, left_length)) = &left_sorted_point[left_index];
                let &(right_penalty, (right_index_of_components, right_length)) = &right_sorted_point[right_index];

                let length =  (left_length + right_length) as usize + anchor_size;

                if (length < cutoff.minimum_aligned_length) || (length < length_of_start_point) {
                    right_start_index = right_index + 1;
                    if right_start_index < right_sorted_point_count {
                        break 'right_loop;
                    } else {
                        break 'left_loop;
                    }
                } else {
                    let penalty = left_penalty + right_penalty;
                    let penalty_per_scale = PRECISION_SCALE * penalty / length;

                    if (penalty_per_scale <= cutoff.maximum_penalty_per_scale) && (penalty_per_scale < penalty_per_scale_of_start_point) {
                        length_of_start_point = length;
                        penalty_per_scale_of_start_point = penalty_per_scale;

                        let start_point_of_wave_front = StartPointOfWaveFront {
                            left_score: left_penalty,
                            left_index_of_components: left_index_of_components,
                            right_score: right_penalty,
                            right_index_of_components: right_index_of_components,
                        };
                        optional_start_point_of_wave_front = Some(start_point_of_wave_front);
                    }
                }
            }
        }

        optional_start_point_of_wave_front
    }
}

#[derive(Debug)]
pub struct StartPointOfWaveFront {
    pub left_score: usize,
    pub left_index_of_components: usize,
    pub right_score: usize,
    pub right_index_of_components: usize,
}