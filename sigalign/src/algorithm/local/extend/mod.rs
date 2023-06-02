use crate::{
    core::{
        regulators::{
            Penalty, Cutoff,
        },
    },
    results::{
        AlignmentPosition, AlignmentOperations,
    }
};
use super::{
    AnchorTable, Anchor, AnchorIndex,
    WaveFront, WaveFrontScore, BackTraceMarker,
    Extension, SparePenaltyCalculator,
    transform_left_additive_position_to_traversed_anchor_index,
    transform_right_additive_position_to_traversed_anchor_index,
};

mod valid_position_candidate;
pub use valid_position_candidate::Vpc;

// Assuming leftmost anchor
#[inline]
pub fn extend_anchor(
    anchor_table: &AnchorTable,
    anchor: &Anchor,
    pattern_index: u32,
    pattern_size: &u32,
    spare_penalty_calculator: &SparePenaltyCalculator,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    extension_buffer: &mut Vec<Extension>,
) {
    // 1. Init
    // 1.1. Define the range of sequence to extend
    let pattern_count = anchor.pattern_count;
    let anchor_size = pattern_count * pattern_size;

    let left_target_end_index = anchor.target_position;
    let right_target_start_index = left_target_end_index + anchor_size;

    let left_query_end_index = pattern_index * pattern_size;
    let right_query_start_index = left_query_end_index + anchor_size;

    // 2. Extend to the right
    // 2.1. Get slices to extend
    let right_target_slice = &target[right_target_start_index as usize..];
    let right_query_slice = &query[right_query_start_index as usize..];
    // 2.2. Calculate the left spare penalty
    let right_spare_penalty = spare_penalty_calculator.get_right_spare_penalty(pattern_index);
    // 2.3. Extend the side with wave front
    right_wave_front.align_right_to_end_point(
        right_target_slice,
        right_query_slice,
        penalties,
        right_spare_penalty,
    );
    // 2.4. Fill sorted vpc vector buffer
    right_vpc_buffer.clear();
    right_wave_front.fill_sorted_vpc_vector(
        &cutoff.maximum_scaled_penalty_per_length,
        right_vpc_buffer,
    );

    // 3. Extend to the left
    // 3.1. Get slices to extend
    let left_target_slice = &target[..left_target_end_index as usize];
    let left_query_slice = &query[..left_query_end_index as usize];
    // 3.2. Calculate the left spare penalty
    let max_scaled_penalty_delta_of_right = right_vpc_buffer[0].scaled_penalty_delta
        + (anchor_size * cutoff.maximum_scaled_penalty_per_length) as i64
    ;
    let left_spare_penalty = spare_penalty_calculator.get_left_spare_penalty(
        max_scaled_penalty_delta_of_right,
        pattern_index,
    );
    // 3.3. Extend the side with wave front
    left_wave_front.align_left_to_end_point(
        left_target_slice,
        left_query_slice,
        penalties,
        left_spare_penalty,
    );
    // 3.4. Fill sorted vpc vector buffer
    left_vpc_buffer.clear();
    left_wave_front.fill_sorted_vpc_vector(
        &cutoff.maximum_scaled_penalty_per_length,
        left_vpc_buffer,
    );
    
    // 4. Backtrace from optimal end points
    // 4.1. Find the optimal end point from vpc vector
    let (optimal_left_vpc_index, optimal_right_vpc_index) = Vpc::get_optimal_position(
        left_vpc_buffer,
        right_vpc_buffer,
        anchor_size * cutoff.maximum_scaled_penalty_per_length,
    );
    // 4.2. Backtrace left
    let left_optimal_vpc = &left_vpc_buffer[optimal_left_vpc_index];
    let left_back_trace_result = left_wave_front.backtrace_of_left_side(
        left_optimal_vpc.penalty,
        *pattern_size,
        left_optimal_vpc.component_index,
        penalties,
        operations_buffer,
        traversed_anchor_index_buffer,
    );
    transform_left_additive_position_to_traversed_anchor_index(
        anchor_table,
        traversed_anchor_index_buffer,
        pattern_index,
        left_target_end_index,
        left_back_trace_result.traversed_anchor_range,
    );
    // 4.2. Backtrace right
    let right_optimal_vpc = &right_vpc_buffer[optimal_right_vpc_index];
    let right_back_trace_result = right_wave_front.backtrace_of_right_side(
        right_optimal_vpc.penalty,
        *pattern_size,
        pattern_count,
        right_optimal_vpc.component_index,
        penalties,
        operations_buffer,
        traversed_anchor_index_buffer,
    );
    transform_right_additive_position_to_traversed_anchor_index(
        anchor_table,
        traversed_anchor_index_buffer,
        pattern_index,
        left_target_end_index,
        right_back_trace_result.traversed_anchor_range,
        *pattern_size,
    );
    
    // 5. Push extension
    let extension = Extension {
        alignment_position: AlignmentPosition {
            query: (
                left_query_end_index - left_back_trace_result.processed_length.0,
                left_query_end_index + right_back_trace_result.processed_length.0,
            ),
            target: (
                left_target_end_index - left_back_trace_result.processed_length.1,
                left_target_end_index + right_back_trace_result.processed_length.1,
            ),
        },
        penalty: left_back_trace_result.penalty_of_extension + right_back_trace_result.penalty_of_extension,
        length: left_back_trace_result.length_of_extension + right_back_trace_result.length_of_extension,
        left_side_operation_range: left_back_trace_result.operation_buffer_range,
        left_traversed_anchor_range: left_back_trace_result.traversed_anchor_range,
        right_side_operation_range: right_back_trace_result.operation_buffer_range,
        right_traversed_anchor_range: right_back_trace_result.traversed_anchor_range,
    };
    extension_buffer.push(extension);
}
