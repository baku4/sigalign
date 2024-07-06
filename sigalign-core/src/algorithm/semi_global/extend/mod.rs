use crate::{
    core::regulators::{
        Penalty, Cutoff,
    },
    results::{
        AlignmentPosition, AlignmentOperations,
    },
};
use super::{
    AnchorTable, AnchorIndex,
    WaveFront, BackTraceMarker, BackTraceResult, TraversedAnchor,
    Extension,
    SparePenaltyCalculator,
    transform_right_additive_positions_to_traversed_anchor_index,
};

mod backtrace;

#[derive(Debug, Clone)]
pub struct SemiGlobalExtension {
    pub penalty: u32,
    pub length: u32,
    pub alignment_position: AlignmentPosition,
    pub left_side_operation_range: (u32, u32),
    pub left_traversed_anchor_range: (u32, u32),
    pub right_side_operation_range: (u32, u32),
    pub right_traversed_anchor_range: (u32, u32),
}

// Assuming leftmost anchor
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
    wave_front.align_right_to_end_point(
        right_target_slice,
        right_query_slice,
        penalties,
        right_spare_penalty,
    );
    // 2.4. Check if invalid
    //   - confirm invalid: early drop here
    if !wave_front.is_reached_to_sequence_end() {
        let (penalty, component_index) = get_the_last_end_point(wave_front);
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
    //   - have chance to valid: proceed
    let right_back_trace_result = wave_front.backtrace_from_the_end_of_right_side(
        *pattern_size,
        cutoff.maximum_scaled_penalty_per_length as i32,
        pattern_count,
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
            (right_back_trace_result.length_of_extension * cutoff.maximum_scaled_penalty_per_length) as i64
            - right_back_trace_result.penalty_of_extension as i64
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
    if !wave_front.is_reached_to_sequence_end() {
        return None;
    }
    //   - have chance to valid: proceed
    let (left_back_trace_result, leftmost_anchor_index) = wave_front.backtrace_from_the_end_of_left_side(
        *pattern_size,
        penalties,
        operations_buffer,
        anchor_table,
        anchor_index.0,
        left_target_end_index,
    )?;
    let leftmost_anchor_index = leftmost_anchor_index.unwrap_or(anchor_index);
    
    // 4. Check validation
    let penalty = left_back_trace_result.penalty_of_extension + right_back_trace_result.penalty_of_extension;
    let length = left_back_trace_result.length_of_extension + right_back_trace_result.length_of_extension;
    
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
        penalty,
        length,
        left_side_operation_range: left_back_trace_result.operation_buffer_range,
        right_side_operation_range: right_back_trace_result.operation_buffer_range,
        leftmost_anchor_index,
        right_operation_meet_edge: true, // Always true in semi-global
    };
    return Some(extension);
}

#[inline(always)]
fn get_the_last_end_point(
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
