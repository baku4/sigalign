use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

use std::cmp::Ordering;

mod valid_position_candidate;
use valid_position_candidate::VPC;
mod extend;
use extend::LocalExtension;

pub fn local_alignment_algorithm<S: SequenceProvider>(
    reference: &Reference<S>,
    sequence_buffer: &mut S::Buffer,
    query: Sequence,
    pattern_size: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> AlignmentResult {
    let pos_table_map = PosTable::new_by_record(&reference, &query, pattern_size);

    let record_alignment_results: Vec<RecordAlignmentResult> = pos_table_map.into_iter().filter_map(|(record_index, pos_table)| {
        reference.fill_sequence_buffer(record_index, sequence_buffer);
        let record_sequence = sequence_buffer.request_sequence();
        let anchor_alignment_results = local_alignment_query_to_record(
            &pos_table,
            pattern_size,
            record_sequence,
            &query,
            &penalties,
            &cutoff,
            left_wave_front,
            right_wave_front,
        );

        if anchor_alignment_results.len() == 0 {
            None
        } else {
            Some(RecordAlignmentResult {
                index: record_index,
                alignments: anchor_alignment_results,
            })
        }
    }).collect();

    AlignmentResult(record_alignment_results)
}

fn local_alignment_query_to_record(
    pos_table: &PosTable,
    pattern_size: usize,
    record_sequence: Sequence,
    query_sequence: Sequence,
    penalties: &Penalties,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> Vec<AnchorAlignmentResult> {
    let sorted_anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
        (0..pattern_position.len()).map(move |anchor_index| {
            (pattern_index, anchor_index)
        })
    }).flatten().collect();

    let mut anchor_table: Vec<Vec<Anchor>> = pos_table.0.iter().map(|pattern_position| {
        vec![Anchor::new_empty(); pattern_position.len()]
    }).collect();

    let mut local_alignments: Vec<LocalAlignment> = Vec::new();
    sorted_anchor_indices.into_iter().for_each(|current_anchor_index| {
        let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];
    
        if !current_anchor.registered {
            //
            // (1) Get extension of current anchor
            //
            let cached_extension = current_anchor.extensions_cache.take();
            let (
                left_extension,
                right_extension,
                left_traversed_anchors,
                right_traversed_anchors,
                left_scaled_penalty_margins,
                right_scaled_penalty_margins,
            ) = match cached_extension {
                Some(v) => {
                    (
                        v.left_extension,
                        v.right_extension,
                        v.left_traversed_anchors,
                        v.right_traversed_anchors,
                        v.left_scaled_penalty_margins,
                        v.right_scaled_penalty_margins,
                    )
                },
                None => {
                    let scaled_penalty_margin_of_left = ((pattern_size - 1) * cutoff.maximum_penalty_per_scale) as i64; // Assuming this anchor is leftmost of alignment (It is safe)
                    let local_extension = pos_table.extend_with_right_first(
                        &current_anchor_index,
                        pattern_size,
                        record_sequence,
                        query_sequence,
                        penalties,
                        cutoff,
                        scaled_penalty_margin_of_left,
                        left_wave_front,
                        right_wave_front,
                    );
                    (
                        local_extension.left_extension,
                        local_extension.right_extension,
                        local_extension.left_traversed_anchors,
                        local_extension.right_traversed_anchors,
                        local_extension.left_scaled_penalty_margins,
                        local_extension.right_scaled_penalty_margins,
                    )
                }
            };

            //
            // (2) Make symbol
            //
            let mut symbol = Vec::<AnchorIndex>::with_capacity(
                left_traversed_anchors.len() + right_traversed_anchors.len() + 1
            );
            left_traversed_anchors.iter().for_each(|traversed_anchors| {
                symbol.push(traversed_anchors.anchor_index.clone())
            });
            symbol.push(current_anchor_index.clone());
            right_traversed_anchors.iter().rev().for_each(|traversed_anchors| {
                symbol.push(traversed_anchors.anchor_index.clone())
            });

            //
            // (3) Make Local Alignment
            //
            let anchor_position = &pos_table.0[current_anchor_index.0][current_anchor_index.1];
            let pattern_count = anchor_position.pattern_count;
            let anchor_size = pattern_count * pattern_size;

            let length = left_extension.length + right_extension.length + anchor_size;
            let query_length = length - left_extension.insertion_count as usize - right_extension.insertion_count as usize;
            let penalty = left_extension.penalty + right_extension.penalty;
            let valid_anchor_alignment_operations_and_position = if length >= cutoff.minimum_aligned_length {
                let anchor_query_position = current_anchor_index.0 * pattern_size;
                let anchor_record_position = anchor_position.record_position;
                let alignment_position = AlignmentPosition {
                    record: (
                        anchor_record_position + left_extension.deletion_count as usize - left_extension.length,
                        anchor_record_position + anchor_size + right_extension.length - right_extension.deletion_count as usize,
                    ),
                    query: (
                        anchor_query_position + left_extension.insertion_count as usize - left_extension.length,
                        anchor_query_position + anchor_size + right_extension.length - right_extension.insertion_count as usize,
                    ),
                };

                let alignment_operations = AlignmentOperation::concatenate_operations(
                   left_extension.operations,
                   right_extension.operations,
                   anchor_size as u32,
                );
                Some((alignment_operations, alignment_position))
            } else {
                None
            };

            //
            // (4) Find optimal anchors for this local alignment
            //
            let representative_symbol_index = left_traversed_anchors.len();
            let mut optional_leftmost_symbol_index = None;
            let mut optional_rightmost_symbol_index = None;
            // For left symbols
            for index in 0..left_traversed_anchors.len() {
                let left_traversed_anchor = &left_traversed_anchors[index];
                let left_traversed_anchor_index = left_traversed_anchor.anchor_index;

                let left_anchor = &mut anchor_table[left_traversed_anchor_index.0][left_traversed_anchor_index.1];

                if left_anchor.registered {
                    // Left anchor's optimal alignment is other
                    continue
                } else {
                    // If left anchor does not have alignment cache: extend
                    if left_anchor.extensions_cache.is_none() {
                        let left_scaled_penalty_margin = left_scaled_penalty_margins[index];
                        let local_alignment_of_traversed = pos_table.extend_with_right_first(
                            &left_traversed_anchor_index,
                            pattern_size,
                            record_sequence,
                            query_sequence,
                            penalties,
                            cutoff,
                            left_scaled_penalty_margin,
                            left_wave_front,
                            right_wave_front,
                        );
                        left_anchor.extensions_cache = Some(local_alignment_of_traversed);
                    };
                    let local_extension_of_left_anchor = left_anchor.extensions_cache.as_ref().unwrap();
                    let search_result = local_extension_of_left_anchor.right_traversed_anchors.binary_search_by(|traversed_anchor| {
                        traversed_anchor.anchor_index.cmp(&current_anchor_index)
                    });
                    if search_result.is_ok() {
                        let leftmost_symbol_index = index;
                        optional_leftmost_symbol_index = Some(leftmost_symbol_index);
                        break;
                    }
                }
            }
            // For right symbols
            for index in 0..right_traversed_anchors.len() {
                let right_traversed_anchor = &right_traversed_anchors[index];
                let right_traversed_anchor_index = right_traversed_anchor.anchor_index;

                let right_anchor = &mut anchor_table[right_traversed_anchor_index.0][right_traversed_anchor_index.1];

                if right_anchor.registered {
                    // Left anchor's optimal alignment is other
                    continue
                } else {
                    // If left anchor does not have alignment cache: extend
                    if right_anchor.extensions_cache.is_none() {
                        let right_scaled_penalty_margin = right_scaled_penalty_margins[index];
                        let local_alignment_of_traversed = pos_table.extend_with_left_first(
                            &right_traversed_anchor_index,
                            pattern_size,
                            record_sequence,
                            query_sequence,
                            penalties,
                            cutoff,
                            right_scaled_penalty_margin,
                            left_wave_front,
                            right_wave_front,
                        );
                        right_anchor.extensions_cache = Some(local_alignment_of_traversed);
                    };
                    let local_extension_of_right_anchor = right_anchor.extensions_cache.as_ref().unwrap();
                    let search_result = local_extension_of_right_anchor.left_traversed_anchors.binary_search_by(|traversed_anchor| {
                        traversed_anchor.anchor_index.cmp(&current_anchor_index)
                    });
                    if search_result.is_ok() {
                        let rightmost_symbol_index = symbol.len() - index - 1;
                        optional_rightmost_symbol_index = Some(rightmost_symbol_index);
                        break;
                    }
                }
            }
            
            let leftmost_optimal_symbol_index = match optional_leftmost_symbol_index {
                Some(v) => v,
                None => representative_symbol_index,
            };
            let rightmost_optimal_symbol_index = match optional_rightmost_symbol_index {
                Some(v) => v,
                None => representative_symbol_index,
            };

            let mut non_optimal_anchor_indices = Vec::new();
            symbol[..leftmost_optimal_symbol_index].iter().for_each(|&anchor_index| {
                non_optimal_anchor_indices.push(anchor_index);
            });
            symbol[rightmost_optimal_symbol_index+1..].iter().for_each(|&anchor_index| {
                non_optimal_anchor_indices.push(anchor_index);
            });
            // Register anchors
            symbol[leftmost_optimal_symbol_index..=rightmost_optimal_symbol_index].iter().for_each(|&anchor_index| {
                anchor_table[anchor_index.0][anchor_index.1].registered = true;
            });

            let local_alignment = LocalAlignment {
                // Symbol
                symbol,
                // Length and penalty
                query_length,
                penalty,
                length,
                // About operation
                representative_symbol_index,
                valid_anchor_alignment_operations_and_position,
                left_traversed_anchors,
                right_traversed_anchors,
                // About Optimum
                leftmost_optimal_symbol_index,
                rightmost_optimal_symbol_index,
                non_optimal_anchor_indices,
            };

            local_alignments.push(local_alignment);
        }
    });

    // Sort by
    // (1) longer query is left
    // (2) lesser penalty is left
    local_alignments.sort_unstable_by(|a, b| {
        let query_length_cmp = a.query_length.partial_cmp(&b.query_length).unwrap();
        match query_length_cmp {
            Ordering::Equal => {
                a.penalty.partial_cmp(&b.penalty).unwrap()
            },
            Ordering::Greater => Ordering::Less,
            Ordering::Less => Ordering::Greater,
        }
    });

    local_alignments.into_iter().filter_map(|local_alignment| {
        match local_alignment.valid_anchor_alignment_operations_and_position {
            Some((alignment_operations, alignment_position)) => {
                let mut is_unique_position = true;
                for non_optimal_anchor_index in local_alignment.non_optimal_anchor_indices.into_iter() {
                    if anchor_table[non_optimal_anchor_index.0][non_optimal_anchor_index.1].included {
                        is_unique_position = false;
                        break;
                    }
                }
                if is_unique_position {
                    local_alignment.symbol.into_iter().for_each(|anchor_index| {
                        anchor_table[anchor_index.0][anchor_index.1].included = true;
                    });
                    let anchor_alignment_result = AnchorAlignmentResult {
                        penalty: local_alignment.penalty,
                        length: local_alignment.length,
                        position: alignment_position,
                        operations: alignment_operations,
                    };
                    Some(anchor_alignment_result)
                } else {
                    None
                }
            },
            None => {
                None
            },
        }
    }).collect()
}

struct AnchorTable(Vec<Vec<Anchor>>);

#[derive(Debug, Clone)]
struct Anchor {
    extensions_cache: Option<LocalExtension>,
    registered: bool, // If registered in local alignment
    included: bool, // If included in used symbol
}
impl Anchor {
    fn new_empty() -> Self {
        Self {
            extensions_cache: None,
            registered: false,
            included: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LocalAlignment {
    // Symbol
    symbol: Vec<AnchorIndex>, // sorted anchor indices
    // Length and penalty
    query_length: usize,
    penalty: usize,
    length: usize,
    // About operation
    representative_symbol_index: usize,
    valid_anchor_alignment_operations_and_position: Option<(Vec<AlignmentOperation>, AlignmentPosition)>, // None, if (len < cutoff len)
    left_traversed_anchors: Vec<TraversedAnchor>,
    right_traversed_anchors: Vec<TraversedAnchor>,
    // About Optimum
    leftmost_optimal_symbol_index: usize,
    rightmost_optimal_symbol_index: usize,
    non_optimal_anchor_indices: Vec<AnchorIndex>,
}
