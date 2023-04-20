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
// mod symbol;
// use symbol::{
//     LEFT_EMPTY_SYMBOL,
//     RIGHT_EMPTY_SYMBOL,
//     MERGED_EMPTY_SYMBOL,
// };
mod spare_penalty;
use spare_penalty::{
    SparePenaltyCalculator,
    get_right_spare_penalty_by_pattern_index_from_the_right,
    DepSparePenaltyCalculator,
};
mod extend;
use extend::{
    LocalExtension,
    Vpc,
    extend_assuming_leftmost_anchor,
    
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
    wave_front_for_left_extension: &mut WaveFront,
    wave_front_for_right_extension: &mut WaveFront,
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
    let mut left_sorted_vpc_vector_buffer: Vec<Vpc> = Vec::new();
    let mut right_sorted_vpc_vector_buffer: Vec<Vpc> = Vec::new();
    let mut extension_buffer: Vec<LocalExtension> = Vec::new();
    //   - (2) Reuse without clear, need init and index to operate
    let mut operations_buffer: Vec<AlignmentOperations> = Vec::new();
    // ^

    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor = &mut anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];

        // If skipped: the result of that anchor is included in the result already.
        if !current_anchor.skipped {
            // Extend if not extended
            if !current_anchor.extended {
                // extend_assuming_leftmost_anchor();
            }

            // Check the right traversed anchors
            let right_traversed_anchors: Vec<AnchorIndex> = Vec::new();
            for right_traversed_anchor in right_traversed_anchors.iter().rev() {
                let right_anchor = &mut anchor_table.0[right_traversed_anchor.0 as usize][right_traversed_anchor.1 as usize];
                if !right_anchor.skipped {
                    // Extend if not extended
                    if !right_anchor.extended {
                        // extend_assuming_leftmost_anchor();
                    }

                    let left_traversed_anchors: Vec<AnchorIndex> = Vec::new();
    
                    skip_extending_of_anchors_in_intersection();
                }
            }
        }
    });

    Vec::new()
}

fn skip_extending_of_anchors_in_intersection() {
    //
}
