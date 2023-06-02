use crate::{
    core::{
        regulators::{
            Penalty, Cutoff, PREC_SCALE,
        },
    },
    results::{
        AlignmentPosition, AlignmentOperations,
    }
};
use super::{
    AnchorTable, Anchor, AnchorIndex,
    WaveFront, BackTraceMarker, BackTraceResult,
    Extension,
    SparePenaltyCalculator,
    transform_left_additive_position_to_traversed_anchor_index,
    transform_right_additive_position_to_traversed_anchor_index,
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
    anchor: &Anchor,
    pattern_index: u32,
    pattern_size: &u32,
    spare_penalty_calculator: &SparePenaltyCalculator,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    wave_front: &mut WaveFront,
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
        let right_traversed_anchor_range = wave_front.backtrace_to_get_right_side_traversed_anchor(
            penalty,
            *pattern_size,
            pattern_count,
            component_index,
            penalties,
            traversed_anchor_index_buffer,
        );
        transform_right_additive_position_to_traversed_anchor_index(
            anchor_table,
            traversed_anchor_index_buffer,
            pattern_index,
            left_target_end_index,
            right_traversed_anchor_range,
            *pattern_size,
        );

        let incomplete_extension = Extension {
            alignment_position: AlignmentPosition {
                query: (0, 0),
                target: (0, 0),
            },
            penalty: 0,
            length: 0,
            left_side_operation_range: (0, 0),
            left_traversed_anchor_range: (0, 0),
            right_side_operation_range: (0, 0),
            right_traversed_anchor_range: right_traversed_anchor_range,
        };
        extension_buffer.push(incomplete_extension);

        return;
    }
    //   - have chance to valid: proceed
    let right_back_trace_result = wave_front.backtrace_from_the_end_of_right_side(
        *pattern_size,
        pattern_count,
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
            pattern_index,
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
        let (penalty, component_index) = get_the_last_end_point(wave_front);
        let left_traversed_anchor_range = wave_front.backtrace_to_get_left_side_traversed_anchor(
            penalty,
            *pattern_size,
            component_index,
            penalties,
            traversed_anchor_index_buffer,
        );
        transform_left_additive_position_to_traversed_anchor_index(
            anchor_table,
            traversed_anchor_index_buffer,
            pattern_index,
            left_target_end_index,
            left_traversed_anchor_range,
        );

        let incomplete_extension = Extension {
            alignment_position: AlignmentPosition {
                query: (0, 0),
                target: (0, 0),
            },
            penalty: 0,
            length: 0,
            left_side_operation_range: (0, 0),
            left_traversed_anchor_range: left_traversed_anchor_range,
            right_side_operation_range: (0, 0),
            right_traversed_anchor_range: right_back_trace_result.traversed_anchor_range,
        };
        extension_buffer.push(incomplete_extension);

        return;
    }
    //   - have chance to valid: proceed
    let left_back_trace_result = wave_front.backtrace_from_the_end_of_left_side(
        *pattern_size,
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
    
    // 4. Check validation
    let penalty = left_back_trace_result.penalty_of_extension + right_back_trace_result.penalty_of_extension;
    let mut length = left_back_trace_result.length_of_extension + right_back_trace_result.length_of_extension;
    //   - if invalid: mark the length as 0
    if (
        length < cutoff.minimum_aligned_length
    ) || (
        penalty * PREC_SCALE > cutoff.maximum_scaled_penalty_per_length * length
    ) {
        length = 0;
    };
    
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
        left_traversed_anchor_range: left_back_trace_result.traversed_anchor_range,
        right_side_operation_range: right_back_trace_result.operation_buffer_range,
        right_traversed_anchor_range: right_back_trace_result.traversed_anchor_range,
    };
    extension_buffer.push(extension);
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
