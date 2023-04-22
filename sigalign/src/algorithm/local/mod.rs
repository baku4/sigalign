use crate::core::{
    ReferenceInterface, SequenceBuffer,
    regulators::{
        Penalty, Cutoff,
    }
};
use crate::results::{
    AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
};
use super::{
    WaveFront, WaveFrontScore, BackTraceMarker,
};

mod anchor;
use anchor::{
    Anchor,
    AnchorTable,
};
pub use anchor::{
    AnchorIndex,
};
mod spare_penalty;
pub use spare_penalty::{
    LocalSparePenaltyCalculator,
};
mod extend;
use extend::{
    extend_anchor,
    mark_anchor_as_extended,
};
pub use extend::{
    LocalExtension,
    Vpc,
};
mod skip;
use skip::{
    mark_traversed_anchors_as_skipped
};
mod transform;
use transform::{
    transform_extension_to_result,
};

pub fn local_alignment_algorithm<R: ReferenceInterface>(
    reference: &R,
    sequence_buffer: &mut R::Buffer,
    query: &[u8],
    pattern_size: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    spare_penalty_calculator: &mut LocalSparePenaltyCalculator,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    sorted_anchor_indices: &mut Vec<AnchorIndex>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    extension_buffer: &mut Vec<LocalExtension>,
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
            spare_penalty_calculator,
            left_wave_front,
            right_wave_front,
            left_vpc_buffer,
            right_vpc_buffer,
            sorted_anchor_indices,
            traversed_anchor_index_buffer,
            operations_buffer,
            extension_buffer,
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
    // Buffers
    spare_penalty_calculator: &mut LocalSparePenaltyCalculator,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    sorted_anchor_indices: &mut Vec<AnchorIndex>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    extension_buffer: &mut Vec<LocalExtension>,
) -> Vec<AnchorAlignmentResult> {
    // Initialize
    //   - Clear the buffers
    sorted_anchor_indices.clear();
    traversed_anchor_index_buffer.clear();
    operations_buffer.clear();
    extension_buffer.clear();
    //   - Change the last pattern index
    spare_penalty_calculator.change_last_pattern_index(
        anchor_table.0.len() as u32 - 1
    );
    //   - Create vector of results
    let mut anchor_alignment_results: Vec<AnchorAlignmentResult> = Vec::new();
    //   - Fill the anchor index vecotr
    anchor_table.0.iter().enumerate().for_each(|(pattern_index, anchors_of_pattern)| {
        (0..anchors_of_pattern.len() as u32).for_each(|v| {
            sorted_anchor_indices.push((pattern_index as u32, v))
        })
    });

    // Run
    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor = &anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
        // If skipped: the result of that anchor is included in the result already.
        if !current_anchor.skipped {
            // (1) Extend the current anchor
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
                    left_vpc_buffer,
                    right_vpc_buffer,
                    operations_buffer,
                    traversed_anchor_index_buffer,
                    extension_buffer,
                );
                mark_anchor_as_extended(
                    current_anchor,
                    extension_buffer.len() as u32 -1,
                );
            }

            // (2) Check the all right traversed anchors
            let right_traversed_anchor_index_range = extension_buffer[
                current_anchor.extension_index as usize
            ].right_traversed_anchor_range;            
            (right_traversed_anchor_index_range.0..right_traversed_anchor_index_range.1).for_each(|idx: u32| {
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
                            left_vpc_buffer,
                            right_vpc_buffer,
                            operations_buffer,
                            traversed_anchor_index_buffer,
                            extension_buffer,
                        );
                        mark_anchor_as_extended(
                            traversed_anchor,
                            extension_buffer.len() as u32 - 1,
                        );
                    }
                    let extension_of_traversed_anchor = &extension_buffer[
                        traversed_anchor.extension_index as usize
                    ];
                    let left_traversed_anchor_index_range = extension_of_traversed_anchor.left_traversed_anchor_range;
                    

                    mark_traversed_anchors_as_skipped(
                        anchor_table,
                        traversed_anchor_index_buffer,
                        current_anchor_index,
                        idx,
                        right_traversed_anchor_index_range.1,
                        left_traversed_anchor_index_range.0,
                        left_traversed_anchor_index_range.1,
                    );
                }
            });

            // (3) Output result
            let extension_of_current_anchor = &extension_buffer[
                current_anchor.extension_index as usize
            ];
            if extension_of_current_anchor.length >= cutoff.minimum_aligned_length {
                let result = transform_extension_to_result(
                    extension_of_current_anchor,
                    operations_buffer,
                );
                anchor_alignment_results.push(result);
            }
        }
    });

    anchor_alignment_results
}
