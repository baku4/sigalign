use crate::{
    core::{
        ReferenceInterface, SequenceBuffer,
        regulators::{
            Penalty, Cutoff, MinPenaltyForPattern,
        }
    },
    results::{
        AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult,
        AlignmentPosition, AlignmentOperations,
    },
};
use super::{
    Anchor, AnchorTable, AnchorIndex,
    WaveFront, WaveFrontScore, BackTraceMarker, BackTraceResult,
    Extension,
    mark_anchor_as_extended,
    mark_traversed_anchors_as_skipped,
    transform_left_additive_position_to_traversed_anchor_index,
    transform_right_additive_position_to_traversed_anchor_index,
};

mod spare_penalty;
pub use spare_penalty::{
    SemiGlobalSparePenaltyCalculator,
};
mod extend;
use extend::{
    extend_anchor,
};

pub fn semi_global_alignment_algorithm<R: ReferenceInterface>(
    reference: &R,
    sequence_buffer: &mut R::Buffer,
    query: &[u8],
    pattern_size: u32,
    penalties: &Penalty,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    cutoff: &Cutoff,
    wave_front: &mut WaveFront,
) -> AlignmentResult {
    let mut anchor_table_map = AnchorTable::new_by_target_index(reference, query, pattern_size);
    let target_alignment_results: Vec<TargetAlignmentResult> = anchor_table_map.iter_mut().filter_map(|(target_index, anchor_table)| {
        reference.fill_buffer(*target_index, sequence_buffer);
        let target = sequence_buffer.request_sequence();
        // let anchor_alignment_results = semi_global_alignment_query_to_target(
        //     anchor_table,
        //     pattern_size,
        //     target,
        //     &query,
        //     &penalties,
        //     &cutoff,
        //     spare_penalty_calculator,
        //     left_wave_front,
        //     right_wave_front,
        //     left_vpc_buffer,
        //     right_vpc_buffer,
        //     traversed_anchor_index_buffer,
        //     operations_buffer,
        //     extension_buffer,
        // );

        // if anchor_alignment_results.len() == 0 {
        //     None
        // } else {
        //     Some(TargetAlignmentResult {
        //         index: *target_index,
        //         alignments: anchor_alignment_results,
        //     })
        // }

        None
    }).collect();

    AlignmentResult(target_alignment_results)
}

fn semi_global_alignment_query_to_target(
    anchor_table: &mut AnchorTable,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    spare_penalty_calculator: &mut SemiGlobalSparePenaltyCalculator,
    wave_front: &mut WaveFront,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    extension_buffer: &mut Vec<Extension>,
) {
    // Initialize
    //   - Clear the buffers
    traversed_anchor_index_buffer.clear();
    operations_buffer.clear();
    extension_buffer.clear();
    //   - Change the last pattern index
    // FIXME:
    //   - Create vector of results
    let mut anchor_alignment_results: Vec<AnchorAlignmentResult> = Vec::new();
    
    anchor_table.0.iter().enumerate().for_each(|(pattern_index, anchors_of_pattern)| {
        anchors_of_pattern.iter().enumerate().for_each(|(anchor_index_in_pattern, current_anchor)| {
            if !current_anchor.skipped {
                // (1) Extend the current anchor
                if !current_anchor.extended {
                    extend_anchor(
                        &anchor_table,
                        current_anchor,
                        pattern_index as u32,
                        &pattern_size,
                        &spare_penalty_calculator,
                        target,
                        query,
                        penalties,
                        cutoff,
                        wave_front,
                        operations_buffer,
                        traversed_anchor_index_buffer,
                        extension_buffer,
                    );
                    mark_anchor_as_extended(
                        current_anchor,
                        extension_buffer.len() as u32 - 1,
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
                                wave_front,
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
                            (pattern_index as u32, anchor_index_in_pattern as u32),
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
                if extension_of_current_anchor.length != 0 {
                    let result = extension_of_current_anchor.parse_anchor_alignment_result(operations_buffer);
                    anchor_alignment_results.push(result);
                }
            }
        });
    });
}
