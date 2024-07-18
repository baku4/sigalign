use crate::{
    core::regulators::{
        Penalty, Cutoff, PREC_SCALE,
    },
    results::{
        AlignmentPosition, AlignmentOperations,
    },
};
use super::{
    AnchorTable, AnchorIndex,
    WaveFront, BackTraceMarker, TraversedAnchor,
    Extension,
    SparePenaltyCalculator,
    transform_right_additive_positions_to_traversed_anchor_index,
};

mod backtrace;

// Return the optional extension of anchor
//  - None if this anchor is
//     - invalid
//        - not meet sequences' end
//        - not satisfy the cutoff
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
    wave_front: &mut WaveFront,
    operations_buffer: &mut Vec<AlignmentOperations>,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
) -> Option<Extension> { // None if already used position or not reached to the end
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
    wave_front.align_right_to_end_point(
        right_target_slice,
        right_query_slice,
        penalties,
        right_spare_penalty,
    );
    // 2.4. Check if invalid
    //   - confirm invalid: early drop here
    let right_end_point = match wave_front.get_optional_end_point() {
        Some(ep) => ep,
        None => {
            let (penalty, component_index) = get_the_point_of_filling_terminated(wave_front);
            wave_front.backtrace_to_get_only_right_traversed_anchors(
                penalty,
                cutoff.maximum_scaled_penalty_per_length as i32,
                *pattern_size,
                pattern_count,
                component_index,
                penalties,
                traversed_anchors_buffer,
            );
            transform_right_additive_positions_to_traversed_anchor_index(
                anchor_table,
                traversed_anchors_buffer,
                anchor_index.0,
                left_target_end_index,
                *pattern_size,
            );
            return None;
        }
    };
    //   - have chance to valid: proceed
    let (right_query_length, right_target_length, right_alignment_length) = wave_front.get_proceed_length(
        right_end_point.0, right_end_point.1
    );
    // 2.5. Get the operations range
    let right_operation_range_in_buffer = wave_front.backtrace_of_right_side_with_checking_traversed(
        right_end_point.0,
        cutoff.maximum_scaled_penalty_per_length as i32,
        *pattern_size,
        pattern_count,
        right_end_point.1,
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
    
    // 3. Extend to the left
    // 3.1. Get slices to extend
    let left_target_slice = &target[..left_target_end_index as usize];
    let left_query_slice = &query[..left_query_end_index as usize];
    // 3.2. Calculate the left spare penalty
    let left_spare_penalty = {
        let max_scaled_penalty_delta_of_right = {
            ((right_alignment_length + anchor_size) * cutoff.maximum_scaled_penalty_per_length) as i32
            - (right_end_point.0 * PREC_SCALE) as i32
        };
        spare_penalty_calculator.get_left_spare_penalty(
            max_scaled_penalty_delta_of_right,
            anchor_index.0,
        )
    };
    // 3.3. Extend the side with wave front
    wave_front.align_left_to_end_point(
        left_target_slice,
        left_query_slice,
        penalties,
        left_spare_penalty,
    );
    // 3.4. Check if invalid
    //   - confirm invalid: early drop here
    let left_end_point = match wave_front.get_optional_end_point() {
        Some(ep) => ep,
        None => {
            return None;
        }
    };
    //   - have chance to valid: proceed
    let (left_query_length, left_target_length, left_alignment_length) = wave_front.get_proceed_length(
        left_end_point.0, left_end_point.1
    );
    //   - Check if this alignment is valid
    let alignment_length = left_alignment_length + right_alignment_length + anchor_size;
    let penalty = left_end_point.0 + right_end_point.0;
    let is_valid = {
        (alignment_length >= cutoff.minimum_length)
        && (cutoff.maximum_scaled_penalty_per_length * alignment_length >= penalty * PREC_SCALE)
    };
    if !is_valid {
        return None;
    }
    // 3.5. Get the operations range
    let left_operation_range_in_buffer = wave_front.backtrace_of_left_side_while_checking_this_anchor_is_leftmost(
        left_end_point.0,
        *pattern_size,
        left_end_point.1,
        penalties,
        operations_buffer,
    )?;

    // 5. Push extension
    let alignment_position = AlignmentPosition {
        query: (
            left_query_end_index - left_query_length,
            right_query_start_index + right_query_length,
        ),
        target: (
            left_target_end_index - left_target_length,
            right_target_start_index + right_target_length,
        ),
    };
    let extension = Extension {
        alignment_position,
        penalty,
        length: alignment_length,
        left_side_operation_range: left_operation_range_in_buffer,
        right_side_operation_range: right_operation_range_in_buffer,
    };
    return Some(extension);
}

#[inline(always)]
fn get_the_point_of_filling_terminated(
    wave_front: &WaveFront,
) -> (u32, u32) { // (penalty, component index) of end point
    let last_penalty = wave_front.end_point.penalty;
    let wfs = &wave_front.wave_front_scores[last_penalty];
    
    let mut max_query_length = 0;
    let mut comp_index_cache = 0;
    wfs.components_by_k.iter().enumerate().for_each(|(comp_index, comp)| {
        if comp.m.bt != BackTraceMarker::Empty {
            let query_length = comp.m.fr + wfs.max_k - comp_index as i32; // Fr - k
            if max_query_length < query_length {
                max_query_length = query_length;
                comp_index_cache = comp_index;
            }
        }
    });

    (last_penalty as u32, comp_index_cache as u32)
}
