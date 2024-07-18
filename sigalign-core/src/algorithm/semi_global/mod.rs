use crate::{
    core::{
        BufferedPatternLocator, SequenceBuffer,
        regulators::{
            Penalty, Cutoff,
        }
    },
    results::{
        QueryAlignment, TargetAlignment, Alignment,
        AlignmentOperations,
    },
};
use super::{
    AnchorTable, AnchorIndex,
    WaveFront, BackTraceMarker, TraversedAnchor,
    Extension, SparePenaltyCalculator,
    transform_right_additive_positions_to_traversed_anchor_index,
};

mod extend;
use extend::extend_anchor;

// Find all semi-global alignments
#[inline]
pub fn semi_global_alignment_algorithm<L: BufferedPatternLocator>(
    pattern_locater: &L,
    sequence_buffer: &mut L::Buffer,
    query: &[u8],
    sorted_target_indices: &[u32],
    pattern_size: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    // Buffers
    wave_front: &mut WaveFront,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
    operations_buffer: &mut Vec<AlignmentOperations>,
) -> QueryAlignment {
    let mut anchor_table_map = AnchorTable::new_by_target_index(pattern_locater, query, sorted_target_indices, pattern_size);
    let target_alignment_results: Vec<TargetAlignment> = anchor_table_map.iter_mut().filter_map(|(target_index, anchor_table)| {
        pattern_locater.fill_buffer(*target_index, sequence_buffer);
        let target = sequence_buffer.buffered_sequence();
        let anchor_alignment_results = semi_global_alignment_query_to_target(
            anchor_table,
            pattern_size,
            target,
            query,
            penalties,
            cutoff,
            spare_penalty_calculator,
            wave_front,
            traversed_anchors_buffer,
            operations_buffer,
        );

        if anchor_alignment_results.is_empty() {
            None
        } else {
            Some(TargetAlignment {
                index: *target_index,
                alignments: anchor_alignment_results,
            })
        }
    }).collect();

    QueryAlignment(target_alignment_results)
}

fn semi_global_alignment_query_to_target(
    anchor_table: &mut AnchorTable,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    wave_front: &mut WaveFront,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
    operations_buffer: &mut Vec<AlignmentOperations>,
) -> Vec<Alignment> {
    // Initialize
    //   - (1) Clear the buffers
    operations_buffer.clear();
    //   - (2) Change the last pattern index
    spare_penalty_calculator.change_last_pattern_index(
        anchor_table.0.len() as u32 - 1
    );
    //   - (3) Create vector of results
    let mut alignment_results: Vec<Alignment> = Vec::new();

    (0..anchor_table.0.len()).for_each(|pattern_index| {
        (0..anchor_table.0[pattern_index].len()).for_each(|anchor_index_in_pattern| {
            let skipped = {
                let anchor = &anchor_table.0[pattern_index][anchor_index_in_pattern];
                anchor.to_skip
            };
            if !skipped {
                // (1) Extend the anchor if not skipped
                let optional_extension = extend_anchor(
                    anchor_table,
                    (pattern_index as u32, anchor_index_in_pattern as u32),
                    &pattern_size,
                    spare_penalty_calculator,
                    target,
                    query,
                    penalties,
                    cutoff,
                    wave_front,
                    operations_buffer,
                    traversed_anchors_buffer,
                );
                // After extension, "traversed_anchors_buffer" is filled with right traversed anchors

                // (2) If extension exists
                //   - Mark skipped anchors:
                //     Semi-global can always check the right traversed anchor.
                //     Right backtracing is always happened.
                //     "traversed_anchors_buffer" is always filled with right traversed anchors.
                traversed_anchors_buffer.iter().for_each(|tv| {
                    if tv.to_skip {
                        anchor_table.0[
                            tv.addt_pattern_index as usize
                        ][
                            tv.addt_target_position as usize
                        ].to_skip = true;
                    }
                });
                //   - Output alignment when extension exists
                if let Some(extension) = optional_extension {
                    let alignment = extension.parse_anchor_alignment_result(operations_buffer);
                    alignment_results.push(alignment);
                }
            }
        });
    });
    alignment_results
}

// Find semi-global alignments with a limit
#[inline]
pub fn semi_global_alignment_algorithm_with_limit<L: BufferedPatternLocator>(
    pattern_locater: &L,
    sequence_buffer: &mut L::Buffer,
    query: &[u8],
    sorted_target_indices: &[u32],
    pattern_size: u32,
    penalties: &Penalty,
    cutoff: &Cutoff,
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    // Buffers
    wave_front: &mut WaveFront,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    // Limit of the number of alignments
    mut limit: u32,
) -> QueryAlignment {
    let mut anchor_table_map = AnchorTable::new_by_target_index(pattern_locater, query, sorted_target_indices, pattern_size);
    let mut target_alignment_results: Vec<TargetAlignment> = Vec::new();

    for (target_index, anchor_table) in anchor_table_map.iter_mut() {
        pattern_locater.fill_buffer(*target_index, sequence_buffer);
        let target = sequence_buffer.buffered_sequence();
        let alignment_results = semi_global_alignment_query_to_target_with_limit(
            anchor_table,
            pattern_size,
            target,
            query,
            penalties,
            cutoff,
            spare_penalty_calculator,
            wave_front,
            traversed_anchors_buffer,
            operations_buffer,
            &mut limit,
        );
        if !alignment_results.is_empty() {
            target_alignment_results.push(TargetAlignment {
                index: *target_index,
                alignments: alignment_results,
            });
        }
        if limit == 0 {
            break;
        }
    }

    QueryAlignment(target_alignment_results)
}

fn semi_global_alignment_query_to_target_with_limit(
    anchor_table: &mut AnchorTable,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    // Buffers
    spare_penalty_calculator: &mut SparePenaltyCalculator,
    wave_front: &mut WaveFront,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
    operations_buffer: &mut Vec<AlignmentOperations>,
    // Limit of the number of alignments
    limit: &mut u32,
) -> Vec<Alignment> {
    // Initialize
    //   - (1) Clear the buffers
    operations_buffer.clear();
    //   - (2) Change the last pattern index
    spare_penalty_calculator.change_last_pattern_index(
        anchor_table.0.len() as u32 - 1
    );
    //   - (3) Create vector of results
    let mut alignment_results: Vec<Alignment> = Vec::new();
 
    for pattern_index in 0..anchor_table.0.len() {
        for anchor_index_in_pattern in 0..anchor_table.0[pattern_index].len() {
            if *limit == 0 {
                return alignment_results;
            }
            let skipped = {
                let anchor = &anchor_table.0[pattern_index][anchor_index_in_pattern];
                anchor.to_skip
            };
            if !skipped {
                // (1) Extend the anchor if not skipped
                let optional_extension = extend_anchor(
                    anchor_table,
                    (pattern_index as u32, anchor_index_in_pattern as u32),
                    &pattern_size,
                    spare_penalty_calculator,
                    target,
                    query,
                    penalties,
                    cutoff,
                    wave_front,
                    operations_buffer,
                    traversed_anchors_buffer,
                );
                // After extension, "traversed_anchors_buffer" is filled with right traversed anchors
                // (2) If extension exists
                //   - Mark skipped anchors:
                //     Semi-global can always check the right traversed anchor.
                //     Right backtracing is always happened.
                //     "traversed_anchors_buffer" is always filled with right traversed anchors.
                traversed_anchors_buffer.iter().for_each(|tv| {
                    if tv.to_skip {
                        anchor_table.0[
                            tv.addt_pattern_index as usize
                        ][
                            tv.addt_target_position as usize
                        ].to_skip = true;
                    }
                });
                //   - Output alignment when extension exists
                if let Some(extension) = optional_extension {
                    let alignment = extension.parse_anchor_alignment_result(operations_buffer);
                    alignment_results.push(alignment);
                    // Reduce the limit
                    *limit -= 1;
                }
            }
        }
    }
    alignment_results
}