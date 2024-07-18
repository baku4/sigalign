use crate::{
    core::regulators::{
        Cutoff, Penalty
    }, results::{
        AlignmentOperations, AlignmentPosition
    }
};
use super::{
    AnchorTable, AnchorIndex,
    WaveFront, WaveFrontScore, BackTraceMarker, TraversedAnchor,
    Extension, SparePenaltyCalculator,
    transform_right_additive_positions_to_traversed_anchor_index,
};

mod valid_position_candidate;
pub use valid_position_candidate::Vpc;

// Return the optional extension of anchor
//  - None if this anchor is
//     - invalid
//     - or not leftmost (= having traversed anchor on the left)
#[inline]
pub fn extend_anchor(
    anchor_table: &AnchorTable,
    anchor_index: AnchorIndex,
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
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
) -> Option<Extension> {
    // 1. Init
    let anchor = &anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
    // 1.1. Define the range of sequence to extend    
    let pattern_count = anchor.pattern_count;
    let anchor_size = pattern_count * pattern_size;

    let left_target_end_index = anchor.target_position;
    let right_target_start_index = left_target_end_index + anchor_size;

    let left_query_end_index = anchor_index.0 * pattern_size;
    let right_query_start_index = left_query_end_index + anchor_size;

    // 2. Extend to the right
    // 2.1. Get slices to extend
    let right_target_slice = &target[right_target_start_index as usize..];
    let right_query_slice = &query[right_query_start_index as usize..];
    // 2.2. Calculate the left spare penalty
    let right_spare_penalty = spare_penalty_calculator.get_right_spare_penalty(anchor_index.0);
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
        + (anchor_size * cutoff.maximum_scaled_penalty_per_length) as i32
    ;
    let left_spare_penalty = spare_penalty_calculator.get_left_spare_penalty(
        max_scaled_penalty_delta_of_right,
        anchor_index.0,
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
    let left_optimal_vpc = &left_vpc_buffer[optimal_left_vpc_index];
    let right_optimal_vpc = &right_vpc_buffer[optimal_right_vpc_index];
    // 4.2. Check if the alignment result is valid before backtracing
    let (alignment_position, alignment_length) = {
        let (q1, t1, a1) = left_wave_front.get_proceed_length(left_optimal_vpc.penalty, left_optimal_vpc.component_index);
        let (q2, t2, a2) = right_wave_front.get_proceed_length(right_optimal_vpc.penalty, right_optimal_vpc.component_index);
        (
            AlignmentPosition {
                query: (
                    left_query_end_index - q1,
                    right_query_start_index + q2,
                ),
                target: (
                    left_target_end_index - t1,
                    right_target_start_index + t2,
                ),
            },
            a1 + a2 + anchor_size,
        )
    };
    if alignment_length < cutoff.minimum_length {
        return None
    }
    // 4.3. Backtrace from left
    //   - None if this anchor is not leftmost (= having traversed anchor on the left)
    let left_operation_range_in_buffer = left_wave_front.backtrace_of_left_side_while_checking_this_anchor_is_leftmost(
        left_optimal_vpc.penalty,
        *pattern_size,
        left_optimal_vpc.component_index,
        penalties,
        operations_buffer,
    )?;
    // 4.4. Backtrace from right
    let right_operation_range_in_buffer = right_wave_front.backtrace_of_right_side_with_checking_traversed(
        right_optimal_vpc.penalty,
        cutoff.maximum_scaled_penalty_per_length as i32,
        *pattern_size,
        pattern_count,
        right_optimal_vpc.component_index,
        penalties,
        operations_buffer,
        traversed_anchors_buffer,
    );
    transform_right_additive_positions_to_traversed_anchor_index(
        anchor_table,
        traversed_anchors_buffer,
        anchor_index.0,
        left_target_end_index,
        *pattern_size,
    );

    // 5. Push extension
    let extension = Extension {
        alignment_position,
        penalty: left_optimal_vpc.penalty + right_optimal_vpc.penalty,
        length: alignment_length,
        left_side_operation_range: left_operation_range_in_buffer,
        right_side_operation_range: right_operation_range_in_buffer,
    };
    Some(extension)
}
