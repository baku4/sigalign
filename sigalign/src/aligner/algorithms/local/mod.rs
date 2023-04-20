use crate::core::{
    ReferenceInterface, SequenceBuffer,
    regulators::{
        Penalty, Cutoff,
    }
};
use crate::results::{
    AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
};
use super::common_steps::{
    Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty,
};
use std::cmp::Ordering;
use ahash::AHashSet;

mod anchor;
use anchor::{
    Anchor,
    AnchorTable,
    AnchorIndex,
};
mod spare_penalty;
use spare_penalty::{
    SparePenaltyCalculator,
};
mod extend;
use extend::{
    LocalExtension,
    extend_anchor,
    Vpc,
};


// mod evaluate;
// use evaluate::{
//     sort_left_side_extensions,
//     sort_right_side_extensions,
// };

pub fn local_alignment_algorithm<R: ReferenceInterface>(
    reference: &R,
    sequence_buffer: &mut R::Buffer,
    query: &[u8],
    pattern_size: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront, 
    right_wave_front: &mut WaveFront,
) -> AlignmentResult {
    let mut anchor_table_map = AnchorTable::new_by_target_index(reference, query, pattern_size);

    let target_alignment_results: Vec<TargetAlignmentResult> = anchor_table_map.iter_mut().filter_map(|(target_index, anchor_table)| {
        reference.fill_buffer(*target_index, sequence_buffer);
        let target = sequence_buffer.request_sequence();
        let anchor_alignment_results = local_alignment_query_to_target(
            anchor_table,
            pattern_size,
            target,
            &query,
            &penalties,
            &cutoff,
            left_wave_front,
            right_wave_front,
        );

        if anchor_alignment_results.len() == 0 {
            None
        } else {
            Some(TargetAlignmentResult {
                index: *target_index,
                alignments: anchor_alignment_results,
            })
        }
    }).collect();

    AlignmentResult(target_alignment_results)
}

#[inline]
fn local_alignment_query_to_target(
    anchor_table: &mut AnchorTable,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> Vec<AnchorAlignmentResult> {
    // TODO: Pre defined
    let max_pattern_count = query.len() as u32 / pattern_size;
    let spare_penalty_calculator = SparePenaltyCalculator::new(
        penalties,
        cutoff.maximum_scaled_penalty_per_length,
        pattern_size,
        max_pattern_count,
    );

    // TODO: Use buffer
    //   - (1) Reuse with clear
    let mut sorted_anchor_indices: Vec<AnchorIndex> = Vec::new();
    let mut left_vpc_buffer: Vec<Vpc> = Vec::new();
    let mut right_vpc_buffer: Vec<Vpc> = Vec::new();
    let mut extension_buffer: Vec<LocalExtension> = Vec::new();
    //   - (2) Reuse without clear, need init and index to operate
    let mut operations_buffer: Vec<AlignmentOperations> = Vec::new();
    let mut traversed_anchor_index_buffer: Vec<AnchorIndex> = Vec::new();
    // ^

    anchor_table.0.iter().enumerate().for_each(|(pattern_index, anchors_of_pattern)| {
        (0..anchors_of_pattern.len() as u32).for_each(|v| {
            sorted_anchor_indices.push((pattern_index as u32, v))
        })
    });
    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor = &anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];

        // If skipped: the result of that anchor is included in the result already.
        if !current_anchor.skipped {
            // Extend if not extended
            if !current_anchor.extended {
                extend_anchor(
                    &anchor_table,
                    current_anchor,
                    current_anchor_index.0,
                    &pattern_size,
                    &spare_penalty_calculator,
                    target,
                    query,
                    penalties,
                    cutoff,
                    left_wave_front,
                    right_wave_front,
                    &mut left_vpc_buffer,
                    &mut right_vpc_buffer,
                    &mut operations_buffer,
                    &mut traversed_anchor_index_buffer,
                    &mut extension_buffer,
                );
                // current_anchor.extension_index = (extension_buffer.len() - 1) as u32; // FIXME: Delete if not needed
            }
            let extension_of_current_anchor = unsafe { extension_buffer.last().unwrap_unchecked() };

            // Check the right traversed anchors
            let right_traversed_anchor_index_range = extension_of_current_anchor.right_traversed_anchor_range;
            (right_traversed_anchor_index_range.1..right_traversed_anchor_index_range.0).rev().for_each(|idx: u32| {
                let traversed_anchor_index = traversed_anchor_index_buffer[idx as usize];
                let traversed_anchor = &anchor_table.0[traversed_anchor_index.0 as usize][traversed_anchor_index.1 as usize];
                if !traversed_anchor.skipped {
                    // Extend if not extended
                    if !traversed_anchor.extended {
                        extend_anchor(
                            &anchor_table,
                            traversed_anchor,
                            traversed_anchor_index.0,
                            &pattern_size,
                            &spare_penalty_calculator,
                            target,
                            query,
                            penalties,
                            cutoff,
                            left_wave_front,
                            right_wave_front,
                            &mut left_vpc_buffer,
                            &mut right_vpc_buffer,
                            &mut operations_buffer,
                            &mut traversed_anchor_index_buffer,
                            &mut extension_buffer,
                        );
                    }
                    let extension_of_traversed_anchor = unsafe { extension_buffer.last().unwrap_unchecked() };
                    let left_traversed_anchor_index_range = extension_of_traversed_anchor.left_traversed_anchor_range;

                    skip_extending_of_anchors_in_intersection();
                }
            });
        }
    });

    Vec::new()
}

#[inline]
fn skip_extending_of_anchors_in_intersection() {
    
}
