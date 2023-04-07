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
    PosTable, AnchorIndex, AnchorPosition, TraversedAnchor,
    Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty,
};
use std::cmp::Ordering;
use ahash::AHashSet;

mod valid_position_candidate;
use valid_position_candidate::{Vpc, VpcIndexPackage};
mod extend;
use extend::LocalExtension;
mod backtrace;

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
    let pos_table_map = PosTable::new_by_target_index(reference, &query, pattern_size);

    let target_alignment_results: Vec<TargetAlignmentResult> = pos_table_map.into_iter().filter_map(|(target_index, pos_table)| {
        reference.fill_buffer(target_index, sequence_buffer);
        let target = sequence_buffer.request_sequence();
        let anchor_alignment_results = local_alignment_query_to_target(
            &pos_table,
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
                index: target_index,
                alignments: anchor_alignment_results,
            })
        }
    }).collect();

    AlignmentResult(target_alignment_results)
}

#[inline]
fn local_alignment_query_to_target(
    pos_table: &PosTable,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> Vec<AnchorAlignmentResult> {
    //FIXME: counter
    let mut skipped_counter = 0;
    let mut right_traversed_anchors_count = 0;

    // TODO: Use buffer
    let mut valid_local_extensions_buffer: Vec<LocalExtension> = Vec::new();
    let sorted_anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
        (0..pattern_position.len()).map(move |anchor_index| {
            (pattern_index as u32, anchor_index as u32)
        })
    }).flatten().collect();
    // println!("# pos table: {:?}", pos_table);
    // println!("# pattern_size: {:?}", pattern_size);
    // println!("# sorted_anchor_indices:\n{:?}", sorted_anchor_indices);
    println!("# total_anchor_count:{}", sorted_anchor_indices.len());

    let mut anchor_table: Vec<Vec<Anchor>> = pos_table.0.iter().map(|pattern_position| {
        vec![Anchor::new_empty(); pattern_position.len()]
    }).collect();

    let scaled_penalty_delta_assuming_on_edge = ((pattern_size - 1) * cutoff.maximum_penalty_per_scale) as i64;

    //
    // 1. Extend all anchors
    //
    sorted_anchor_indices.into_iter().for_each(|current_anchor_index| {
        // println!("# current_anchor_index: {:?}", current_anchor_index);
        let current_anchor = &mut anchor_table[current_anchor_index.0 as usize][current_anchor_index.1 as usize];

        if !current_anchor.skipped {
            //
            // (1) Extend current anchor to the right
            // If cached result is exist -> take it
            //
            let local_extensions = if let Some(v) = current_anchor.extensions_cache.take() {
                v
            } else {
                pos_table.extend_assuming_leftmost_anchor_for_local(
                    &current_anchor_index,
                    pattern_size,
                    target,
                    query,
                    penalties,
                    cutoff,
                    &scaled_penalty_delta_assuming_on_edge,
                    left_wave_front,
                    right_wave_front,
                )
            };
            // Get right traversed anchors
            let leftmost_of_right_extension = &local_extensions[0];
            //unsafe { local_extensions.last().unwrap_unchecked() };
            let right_traversed_anchors = pos_table.get_right_traversed_anchors(
                &current_anchor_index,
                leftmost_of_right_extension,
                pattern_size,
            );
            right_traversed_anchors_count += right_traversed_anchors.len();
            
            // println!("# rightmost_extension: {:?}", rightmost_extension);
            // println!("# right_traversed_anchors: {:?}", right_traversed_anchors);
            //
            // (2) Extend right anchors to the left
            //
            let mut skip_all_other_traversed = false;
            for traversed_anchor_index in right_traversed_anchors.iter().rev() {
                // println!("# traversed_anchor_index: {:?}", traversed_anchor_index);
                let traversed_anchor = &mut anchor_table[traversed_anchor_index.0 as usize][traversed_anchor_index.1 as usize];

                if skip_all_other_traversed { // If the optimal right anchor is found
                    skipped_counter += 1;
                    traversed_anchor.skipped = true;
                } else {
                    let local_extensions_of_right_anchor = if let Some(v) = traversed_anchor.extensions_cache.take() {
                        v
                    } else {
                        pos_table.extend_assuming_rightmost_anchor_for_local(
                            &traversed_anchor_index,
                            pattern_size,
                            target,
                            query,
                            penalties,
                            cutoff,
                            &scaled_penalty_delta_assuming_on_edge,
                            left_wave_front,
                            right_wave_front,
                        )
                    };
                    // println!("# local_extensions_of_right_anchor: {:?}", local_extensions_of_right_anchor);

                    let rightmost_of_left_extension = unsafe { local_extensions_of_right_anchor.last().unwrap_unchecked() };
                    // &local_extensions_of_right_anchor[0];
                    let left_traversed_anchors_of_right_anchor = pos_table.get_left_traversed_anchors(&traversed_anchor_index, rightmost_of_left_extension, pattern_size);
                    // println!("# left_traversed_anchors_of_right_anchor: {:?}", left_traversed_anchors_of_right_anchor);

                    // Check if converged
                    // TODO: Using first index instead of using `contains` method is safe?
                    // (1)
                    // if left_traversed_anchors_of_right_anchor.contains(&current_anchor_index) {
                    // (2)
                    // if (
                    //     !left_traversed_anchors_of_right_anchor.is_empty()
                    //     && left_traversed_anchors_of_right_anchor[0] == current_anchor_index
                    // ) {
                    // (3)
                    // let converged = unsafe {
                    //     leftmost_extension.left_checkpoints.last().unwrap_unchecked()
                    //     == rightmost_extension.left_checkpoints.last().unwrap_unchecked()
                    // };
                    // (4)
                    // let converged = unsafe {
                    //     let cp = rightmost_extension.left_checkpoints.last().unwrap_unchecked();
                    //     leftmost_extension.left_checkpoints.contains(cp)
                    // };
                    // (5)
                    // let converged = unsafe {
                    //     let cp = rightmost_extension.left_checkpoints.last().unwrap_unchecked();
                    //     let mut converged = false;
                    //     for extension in &local_extensions_of_right_anchor {
                    //         if extension.left_checkpoints.contains(cp) {
                    //             converged = true;
                    //             break
                    //         }
                    //     }
                    //     converged
                    // };
                    // (6)
                    let converged = unsafe {
                        let cp = leftmost_of_right_extension.left_checkpoints.last().unwrap_unchecked();
                        let mut converged = false;
                        for extension in &local_extensions_of_right_anchor {
                            if extension.left_checkpoints.last().unwrap_unchecked() == cp {
                                converged = true;
                                break
                            }
                        }
                        converged
                    };
                    if converged {
                        for extension in local_extensions_of_right_anchor {
                            if extension.is_valid(&cutoff.minimum_aligned_length) {
                                valid_local_extensions_buffer.push(extension);
                            }
                        }
                        // To skip all others
                        traversed_anchor.skipped = true;
                        skip_all_other_traversed = true;
                        skipped_counter += 1;
                    } else {
                        traversed_anchor.extensions_cache = Some(local_extensions_of_right_anchor);
                    }
                }
            }
            //
            // (4) Add extensions of current anchor
            //
            for extension in local_extensions {
                if extension.is_valid(&cutoff.minimum_aligned_length) {
                    valid_local_extensions_buffer.push(extension);
                }
            }
        }
    });
    println!("# skipped_counter: {}", skipped_counter);
    println!("# right_traversed_anchors_count:{}", right_traversed_anchors_count);

    //
    // 2. Sort extensions by
    //   - longer query is left
    //   - lesser penalty is left
    //
    // println!("# valid_local_extension_buffer:\n{:?}", valid_local_extensions_buffer);
    valid_local_extensions_buffer.sort_unstable_by(|a, b| {
        let query_length_cmp = a.query_length.partial_cmp(&b.query_length).unwrap();
        match query_length_cmp {
            Ordering::Equal => {
                a.penalty.partial_cmp(&b.penalty).unwrap()
            },
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    });

    //
    // 3. Register extensions to print
    //
    
    // TODO: Use cached buffer
    let mut registered_checkpoints_buffer: AHashSet<(u32, u32)> = AHashSet::new();

    let results = valid_local_extensions_buffer.iter_mut().filter_map(|local_extension| {
        if local_extension.is_already_registered(&registered_checkpoints_buffer) {
            None
        } else {
            local_extension.register_checkpoints(&mut registered_checkpoints_buffer);
            Some(local_extension.to_alignment_result())
        }
    }).collect();
    valid_local_extensions_buffer.clear();
    registered_checkpoints_buffer.clear();

    results
}

#[derive(Debug, Clone)]
struct Anchor {
    // The extension step can be skipped
    skipped: bool,
    // Local extension is sorted from left to right
    extensions_cache: Option<Vec<LocalExtension>>,
}
impl Anchor {
    fn new_empty() -> Self {
        Self {
            skipped: false,
            extensions_cache: None,
        }
    }
}
