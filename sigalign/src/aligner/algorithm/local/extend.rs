use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};
use super::VPC;

#[derive(Debug, Clone)]
pub struct LocalExtension {
    pub left_extension: Extension,
    pub right_extension: Extension,
    pub left_traversed_anchors: Vec<TraversedAnchor>,
    pub right_traversed_anchors: Vec<TraversedAnchor>,
    pub left_scaled_penalty_margins: Vec<i64>,
    pub right_scaled_penalty_margins: Vec<i64>,
}

impl PosTable {
    pub fn extend_right_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_left: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtension {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.record_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_margin = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_left, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - scaled_penalty_margin_of_left;
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_margin);

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &record_sequence[..left_record_last_index];
        let left_query_slice = &query_sequence[..left_query_last_index];

        let right_max_scaled_penalty_margin = right_vpc_vector[0].scaled_penalty_margin as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_margin, anchor_size, left_query_slice.len(), left_record_slice.len(), penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_margin = -anchor_scaled_penalty_margin - right_max_scaled_penalty_margin;
        let left_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_margin);

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = VPC::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_margin, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty margin
        //
        let left_scaled_penalty_margins: Vec<i64> = get_scaled_penalty_margins_of_vpc_vector(
            &left_extension,
            &left_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_margins: Vec<i64> = get_scaled_penalty_margins_of_vpc_vector(
            &right_extension,
            &right_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtension {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_margins,
            right_scaled_penalty_margins,
        }
    }
    pub fn extend_left_first_for_local(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_right: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtension {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.record_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_margin = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &record_sequence[..left_record_last_index];
        let left_query_slice = &query_sequence[..left_query_last_index];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_right, anchor_size, left_query_slice.len(), left_record_slice.len(), penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - scaled_penalty_margin_of_right;
        let left_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_margin);

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let left_max_scaled_penalty_margin = left_vpc_vector[0].scaled_penalty_margin as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_margin, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - left_max_scaled_penalty_margin;
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_margin);

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = VPC::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_margin, anchor_size);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty margin
        //
        let left_scaled_penalty_margins: Vec<i64> = get_scaled_penalty_margins_of_vpc_vector(
            &left_extension,
            &left_vpc_vector,
            cutoff,
            penalties,
            &left_traversed_anchors,
        );

        let right_scaled_penalty_margins: Vec<i64> = get_scaled_penalty_margins_of_vpc_vector(
            &right_extension,
            &right_vpc_vector,
            cutoff,
            penalties,
            &right_traversed_anchors,
        );

        LocalExtension {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_margins,
            right_scaled_penalty_margins,
        }
    }
}

fn get_scaled_penalty_margins_of_vpc_vector(
    extension: &Extension,
    vpc_vector: &Vec<VPC>,
    cutoff: &Cutoff,
    penalties: &Penalties,
    traversed_anchors: &Vec<TraversedAnchor>,
) -> Vec<i64> {
    let scaled_penalty_margin_of_extension = (extension.length * cutoff.maximum_penalty_per_scale) as i64 - (extension.penalty * PRECISION_SCALE) as i64;

    let mut vpc_index_for_traversed_anchor = 0;
    let mut scaled_penalty_margins: Vec<i64> = traversed_anchors.iter().rev().map(|traversed_anchor| {
        let length_to_traversed_start_position = extension.length - traversed_anchor.remained_length;
        let penalty_to_traversed_start_position = extension.penalty - traversed_anchor.remained_penalty;
        let min_query_length = length_to_traversed_start_position - (penalty_to_traversed_start_position / penalties.e);
        while min_query_length > vpc_vector[vpc_index_for_traversed_anchor].query_length {
            vpc_index_for_traversed_anchor += 1;
        }
        let remained_scaled_penalty_margin = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64;
        vpc_vector[vpc_index_for_traversed_anchor].scaled_penalty_margin + remained_scaled_penalty_margin - scaled_penalty_margin_of_extension
    }).collect();
    scaled_penalty_margins.reverse();

    scaled_penalty_margins
}
