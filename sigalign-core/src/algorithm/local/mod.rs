use crate::{
    core::{
        BufferedPatternLocator, SequenceBuffer,
        regulators::{
            Penalty, Cutoff,
        }
    },
    results::{
        AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult,
        AlignmentOperations,
    },
};
use super::{
    AnchorTable, AnchorIndex,
    WaveFront, WaveFrontScore, BackTraceMarker,
    Extension, SparePenaltyCalculator,
    mark_traversed_anchors_as_skipped,
    transform_left_additive_position_to_traversed_anchor_index,
    transform_right_additive_position_to_traversed_anchor_index,
};
mod extend;
use extend::extend_anchor;
pub use extend::Vpc;

#[inline]
pub fn local_alignment_algorithm<L: BufferedPatternLocator>(
    pattern_locater: &L,
    sequence_buffer: &mut L::Buffer,
    query: &[u8],
    sorted_target_indices: &[u32],
    pattern_size: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    // Buffers
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    extension_buffer: &mut Vec<Extension>,
) -> AlignmentResult {
    let mut anchor_table_map = AnchorTable::new_by_target_index(pattern_locater, query, sorted_target_indices, pattern_size);

    let target_alignment_results: Vec<TargetAlignmentResult> = anchor_table_map.iter_mut().filter_map(|(target_index, anchor_table)| {
        pattern_locater.fill_buffer(*target_index, sequence_buffer);
        let target = sequence_buffer.buffered_sequence();
        let anchor_alignment_results = local_alignment_query_to_target(
            anchor_table,
            pattern_size,
            target,
            query,
            penalties,
            cutoff,
            spare_penalty_calculator,
            left_wave_front,
            right_wave_front,
            left_vpc_buffer,
            right_vpc_buffer,
            traversed_anchor_index_buffer,
            operations_buffer,
            extension_buffer,
        );

        if anchor_alignment_results.is_empty() {
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
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
    left_vpc_buffer: &mut Vec<Vpc>,
    right_vpc_buffer: &mut Vec<Vpc>,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    extension_buffer: &mut Vec<Extension>,
) -> Vec<AnchorAlignmentResult> {
    // Initialize
    //   - Clear the buffers
    traversed_anchor_index_buffer.clear();
    operations_buffer.clear();
    extension_buffer.clear();
    //   - Change the last pattern index
    spare_penalty_calculator.change_last_pattern_index(
        anchor_table.0.len() as u32 - 1
    );
    //   - Create vector of results
    let mut anchor_alignment_results: Vec<AnchorAlignmentResult> = Vec::new();

    (0..anchor_table.0.len()).for_each(|pattern_index| {
        (0..anchor_table.0[pattern_index].len()).for_each(|anchor_index_in_pattern| {
            let (skipped, extended) = {
                let anchor = &anchor_table.0[pattern_index][anchor_index_in_pattern];
                (anchor.skipped, anchor.extended)
            };
            if !skipped {
                if !extended {
                    extend_anchor(
                        anchor_table,
                        (pattern_index as u32, anchor_index_in_pattern as u32),
                        &pattern_size,
                        spare_penalty_calculator,
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
                    let current_anchor = &mut anchor_table.0[pattern_index][anchor_index_in_pattern];
                    current_anchor.extended = true;
                    current_anchor.extension_index = extension_buffer.len() as u32 - 1;
                }
                let extension_index_of_current_anchor = anchor_table.0[pattern_index][anchor_index_in_pattern].extension_index as usize;

                // (2) Check the all right traversed anchors
                let right_traversed_anchor_index_range = extension_buffer[extension_index_of_current_anchor].right_traversed_anchor_range;
                (right_traversed_anchor_index_range.0..right_traversed_anchor_index_range.1).for_each(|idx: u32| {
                    let traversed_anchor_index = traversed_anchor_index_buffer[idx as usize];
                    let (traversed_skipped, traversed_extended) = {
                        let anchor = &anchor_table.0[traversed_anchor_index.0 as usize][traversed_anchor_index.1 as usize];
                        (anchor.skipped, anchor.extended)
                    };
                    if !traversed_skipped {
                        // Extend if not extended
                        if !traversed_extended {
                            extend_anchor(
                                anchor_table,
                                traversed_anchor_index,
                                &pattern_size,
                                spare_penalty_calculator,
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
                            let traversed_anchor = &mut anchor_table.0[traversed_anchor_index.0 as usize][traversed_anchor_index.1 as usize];
                            traversed_anchor.extended = true;
                            traversed_anchor.extension_index = extension_buffer.len() as u32 - 1;
                            // mark_anchor_as_extended(
                            //     traversed_anchor,
                            //     extension_buffer.len() as u32 - 1,
                            // );
                        }
                        let extension_index_of_traversed_anchor = anchor_table.0[traversed_anchor_index.0 as usize][traversed_anchor_index.1 as usize].extension_index as usize;
                        let extension_of_traversed_anchor = &extension_buffer[
                            extension_index_of_traversed_anchor
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
                    extension_index_of_current_anchor
                ];
                if extension_of_current_anchor.length >= cutoff.minimum_length {
                    let result = extension_of_current_anchor.parse_anchor_alignment_result(operations_buffer);
                    anchor_alignment_results.push(result);
                }   
            }
        });
    });

    anchor_alignment_results
}
