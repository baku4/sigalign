use crate::core::{
    ReferenceInterface, SequenceBuffer,
    regulators::{
        Penalty, PREC_SCALE, Cutoff, MinPenaltyForPattern,
    },
};
use crate::results::{
    AlignmentResult, TargetAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations, AlignmentOperation,
};

use super::common_steps::{
    PosTable, AnchorIndex, TraversedAnchorDep,
    Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty,
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
    let pos_table_map = PosTable::new_by_target_index(reference, &query, pattern_size);
    let pattern_count = query.len() as u32 / pattern_size;
    let left_penalty_deltas = left_penalty_delta_for_new_pattern(pattern_count, pattern_size, min_penalty_for_pattern, cutoff);

    let record_alignment_results: Vec<TargetAlignmentResult> = pos_table_map.into_iter().filter_map(|(record_index, pos_table)| {
        reference.fill_buffer(record_index, sequence_buffer);
        let record_sequence = sequence_buffer.request_sequence();
        let anchor_alignment_results = semi_global_alignment_query_to_target(
            &pos_table,
            &left_penalty_deltas,
            pattern_size,
            record_sequence,
            &query,
            &penalties,
            &cutoff,
            wave_front,
        );

        if anchor_alignment_results.len() == 0 {
            None
        } else {
            Some(TargetAlignmentResult {
                index: record_index,
                alignments: anchor_alignment_results,
            })
        }
    }).collect();

    AlignmentResult(record_alignment_results)
}

fn semi_global_alignment_query_to_target(
    pos_table: &PosTable,
    left_penalty_delta_for_new_pattern: &Vec<i64>,
    pattern_size: u32,
    target: &[u8],
    query: &[u8],
    penalties: &Penalty,
    cutoff: &Cutoff,
    wave_front: &mut WaveFront,
) -> Vec<AnchorAlignmentResult> {
    let sorted_anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
        (0..pattern_position.len()).map(move |anchor_index| {
            (pattern_index as u32, anchor_index as u32)
        })
    }).flatten().collect();
    
    let mut anchor_table = AnchorTable(
        pos_table.0.iter().map(|pattern_position| {
            vec![Anchor::new_empty(); pattern_position.len()]
        }).collect()
    );

    let mut extension_cache: Vec<SemiGlobalExtension> = Vec::new(); //TODO: Consider cap carefully
    let mut semi_global_alignments = Vec::new();

    sorted_anchor_indices.into_iter().for_each(|current_anchor_index| {
        let current_anchor_is_registered = anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].registered;
        if !current_anchor_is_registered {
            // 
            // (1) Get right extension index
            //
            let have_valid_right_extension = anchor_table.fill_right_extension_index(
                pos_table,
                left_penalty_delta_for_new_pattern,
                &current_anchor_index,
                &mut extension_cache,
                pattern_size,
                target,
                query,
                penalties,
                cutoff,
                wave_front,
            );
            if have_valid_right_extension {
                let right_extension_index = anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].right_extension_index.clone().unwrap();
                //
                // (2) Find rightmost symbol index
                //  While
                //   - Get current + right symbol
                //   - Get whether the rightmost is invalid
                
                let mut rightmost_optimal_symbol_index = 0;
                let mut rightmost_optimal_is_invalid = false;

                let mut right_symbol: Vec<AnchorIndex> = {
                    let right_traversed_anchors = right_extension_index.side_traversed_anchors(&extension_cache);
                    right_traversed_anchors.iter().rev().map(|traversed_anchor| {
                        traversed_anchor.anchor_index.clone()
                    }).collect()
                };

                for (symbol_index, anchor_index) in right_symbol.iter().enumerate().rev() {
                    if rightmost_optimal_symbol_index == 0 { // Rightmost is unknown
                        let have_valid_left_extension = anchor_table.fill_left_extension_index(
                            pos_table,
                            anchor_index,
                            &mut extension_cache,
                            pattern_size,
                            target,
                            query,
                            penalties,
                            cutoff,
                            wave_front,
                        );
                        if have_valid_left_extension { // Success
                            let left_extension_index_of_traversed_anchor = anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize].left_extension_index.as_ref().unwrap();
                            let left_traversed_anchors_of_traversed_anchor = left_extension_index_of_traversed_anchor.side_traversed_anchors(&extension_cache);
                            for traversed_anchor in left_traversed_anchors_of_traversed_anchor {
                                if traversed_anchor.anchor_index == current_anchor_index {
                                    anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize].registered = true;
                                    rightmost_optimal_symbol_index = symbol_index + 1;
                                    break;
                                }
                            }
                        } else { // Fail
                            let anchor = &mut anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
                            anchor.registered = true;
                            anchor.checked_to_invalid = true;
                            rightmost_optimal_symbol_index = symbol_index + 1; // Add current anchor to index 0 in right symbol later.
                            rightmost_optimal_is_invalid = true;
                        }
                    } else { // Rightmost is known
                        let anchor = &mut anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
                        anchor.registered = true;
                        if rightmost_optimal_is_invalid {
                            anchor.checked_to_invalid = true;
                        }
                    }
                };

                //
                // (3) Get left extension index
                //
                let have_valid_left_extension = if rightmost_optimal_is_invalid {
                    let current_anchor = &mut anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    current_anchor.registered = true;
                    current_anchor.checked_to_invalid = true;
                    false
                } else {
                    if rightmost_optimal_symbol_index == 0 {
                        let left_extension_success = anchor_table.fill_left_extension_index(pos_table, &current_anchor_index, &mut extension_cache, pattern_size, target, query, penalties, cutoff, wave_front);
                        left_extension_success
                    } else {
                        true
                    }
                };

                if have_valid_left_extension {
                    // 
                    // (4) Get semi-global alignment
                    //
                    let left_extension_index = anchor_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].left_extension_index.clone().unwrap();

                    let mut symbol: Vec<AnchorIndex> = {
                        let left_traversed_anchors = left_extension_index.side_traversed_anchors(&extension_cache);
                        left_traversed_anchors.iter().map(|traversed_anchor| {
                            traversed_anchor.anchor_index.clone()
                        }).collect()
                    };
                    let representative_symbol_index = symbol.len();
                    let leftmost_optimal_symbol_index = representative_symbol_index;
                    rightmost_optimal_symbol_index += representative_symbol_index;

                    symbol.push(current_anchor_index.clone());
                    symbol.append(&mut right_symbol);

                    let valid_anchor_alignment_operations_and_position = SemiGlobalExtensionIndex::valid_anchor_alignment_operations_and_position(
                        left_extension_index,
                        right_extension_index,
                        pos_table,
                        &current_anchor_index,
                        pattern_size,
                        &extension_cache,
                        cutoff,
                    );

                    match valid_anchor_alignment_operations_and_position {
                        Some((penalty, length, alignment_operations, alignment_position)) => {
                            symbol[leftmost_optimal_symbol_index..=rightmost_optimal_symbol_index].iter().for_each(|&anchor_index| {
                                let anchor = &mut anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
                                anchor.registered = true;
                            });

                            let mut non_optimal_anchor_indices = Vec::new();
                            symbol[..leftmost_optimal_symbol_index].iter().for_each(|&anchor_index| {
                                non_optimal_anchor_indices.push(anchor_index);
                            });
                            symbol[rightmost_optimal_symbol_index+1..].iter().for_each(|&anchor_index| {
                                non_optimal_anchor_indices.push(anchor_index);
                            });

                            semi_global_alignments.push(SemiGlobalAlignment {
                                symbol, // sorted anchor indices
                                penalty,
                                length,
                                alignment_operations,
                                alignment_position,
                                non_optimal_anchor_indices,
                            });
                        },
                        None => {
                            symbol[leftmost_optimal_symbol_index..=rightmost_optimal_symbol_index].iter().for_each(|&anchor_index| {
                                let anchor = &mut anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
                                anchor.registered = true;
                                anchor.checked_to_invalid = true;
                            });
                        },   
                    }
                }
            }
        }
    });

    // Sort by
    // lesser penalty is left
    semi_global_alignments.sort_unstable_by(|a, b| {
        a.penalty.cmp(&b.penalty)
    });

    semi_global_alignments.into_iter().filter_map(|semi_global_alignment| {
        let mut is_unique_position = true;
        for non_optimal_anchor_index in semi_global_alignment.non_optimal_anchor_indices.into_iter() {
            if anchor_table.0[non_optimal_anchor_index.0 as usize][non_optimal_anchor_index.1 as usize].included {
                is_unique_position = false;
                break;
            }
        }
        if is_unique_position {
            semi_global_alignment.symbol.into_iter().for_each(|anchor_index| {
                anchor_table.0[anchor_index.0 as usize][anchor_index.1 as usize].included = true;
            });
            let anchor_alignment_result = AnchorAlignmentResult {
                penalty: semi_global_alignment.penalty,
                length: semi_global_alignment.length,
                position: semi_global_alignment.alignment_position,
                operations: semi_global_alignment.alignment_operations,
            };
            Some(anchor_alignment_result)
        } else {
            None
        }
    }).collect()
}

fn left_penalty_delta_for_new_pattern(
    pattern_count: u32,
    pattern_size: u32,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    cutoff: &Cutoff,
) -> Vec<i64> {
    (0..pattern_count).map(|left_pattern_count| {
        let mut min_penalty = (left_pattern_count / 2) * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even);
        min_penalty += (left_pattern_count % 2) * min_penalty_for_pattern.odd;
        let then_max_length =  pattern_size * left_pattern_count + left_pattern_count;
        (then_max_length * cutoff.maximum_scaled_penalty_per_length - min_penalty) as i64
    }).collect()
}

#[derive(Debug)]
struct AnchorTable(Vec<Vec<Anchor>>);

impl AnchorTable {
    // Return whether extension is success
    fn fill_right_extension_index(
        &mut self,
        pos_table: &PosTable,
        left_penalty_delta_for_new_pattern: &Vec<i64>,
        current_anchor_index: &AnchorIndex,
        extension_cache: &mut Vec<SemiGlobalExtension>,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) -> bool {
        if self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].right_extension_index.is_none() {
            // If have right minimum penalty
            // : Extend left first
            if let Some(right_minimum_penalty) = self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].right_minimum_penalty.clone() {
                let right_scaled_penalty_delta = {
                    let anchor_position = &pos_table.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    let pattern_count = anchor_position.pattern_count;
                    let anchor_size = pattern_count * pattern_size;

                    let left_record_end_index = anchor_position.target_position;
                    let left_query_end_index = current_anchor_index.0 * pattern_size;

                    let then_right_minimum_length = (target.len() as u32 - left_record_end_index).min(
                        query.len() as u32 - left_query_end_index
                    ) - anchor_size;

                    let max_gap = match right_minimum_penalty.checked_sub(penalties.o) {
                        Some(v) => v / penalties.e,
                        None => 0,
                    };
                    let then_right_maximum_length = then_right_minimum_length + max_gap;

                    (then_right_maximum_length * cutoff.maximum_scaled_penalty_per_length) as i64 - (right_minimum_penalty * PREC_SCALE) as i64
                };

                let (optional_left_extension, left_traversed_anchors) = pos_table.extend_left(
                    &current_anchor_index,
                    pattern_size,
                    target,
                    query,
                    penalties,
                    cutoff,
                    right_scaled_penalty_delta,
                    wave_front,
                );

                match optional_left_extension {
                    Some(left_extension) => {
                        let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                        current_anchor.left_extension_index = Some(SemiGlobalExtensionIndex::Owned(extension_cache.len() as u32));
                        extension_cache.push(SemiGlobalExtension(left_extension, left_traversed_anchors));
                    },
                    None => {
                        let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                        current_anchor.registered = true;
                        current_anchor.checked_to_invalid = true;
                        return false
                    },
                }
            };

            // Scaled penalty delta of left
            let scaled_penalty_delta_of_left = match self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].left_extension_index {
                Some(SemiGlobalExtensionIndex::Owned(extension_index)) => {
                    let extension = &extension_cache[extension_index as usize].0;
                    (extension.length * cutoff.maximum_scaled_penalty_per_length) as i64 - (extension.penalty * PREC_SCALE) as i64
                },
                Some(SemiGlobalExtensionIndex::Traversed(extension_index, traversed_anchor_index)) => {
                    let traversed_anchor = &extension_cache[extension_index as usize].1[traversed_anchor_index as usize];
                    (traversed_anchor.remained_length * cutoff.maximum_scaled_penalty_per_length) as i64 - (traversed_anchor.remained_penalty * PREC_SCALE) as i64
                },
                None => {
                    left_penalty_delta_for_new_pattern[current_anchor_index.0 as usize]
                },
            };
            // Generate right extension
            let (optional_right_extension, right_traversed_anchor) = pos_table.extend_right(
                &current_anchor_index,
                pattern_size,
                target,
                query,
                penalties,
                cutoff,
                scaled_penalty_delta_of_left,
                wave_front,
            );
            // Add right extension to extension cache
            match optional_right_extension {
                // If successfully right extended
                Some(right_extension) => {
                    let extension_index = extension_cache.len();
                    right_traversed_anchor.iter().enumerate().for_each(|(traversed_anchor_index, traversed_anchor)| {
                        let anchor_index = &traversed_anchor.anchor_index;
                        let mut anchor = &mut self.0[anchor_index.0 as usize][anchor_index.1 as usize];
                        anchor.right_extension_index = Some(SemiGlobalExtensionIndex::Traversed(extension_index as u32, traversed_anchor_index as u32));
                    });
                    extension_cache.push(SemiGlobalExtension(right_extension, right_traversed_anchor));
                    let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    current_anchor.right_extension_index = Some(SemiGlobalExtensionIndex::Owned(extension_index as u32));
                },
                // If right extension is failed
                None => {
                    right_traversed_anchor.iter().for_each(|traversed_anchor| {
                        let anchor_index = &traversed_anchor.anchor_index;
                        self.0[anchor_index.0 as usize][anchor_index.1 as usize].right_minimum_penalty = Some(traversed_anchor.remained_penalty);
                    });
                    let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    current_anchor.registered = true;
                    current_anchor.checked_to_invalid = true;
                    return false
                },
            }
        }
        true
    }
    fn fill_left_extension_index(
        &mut self,
        pos_table: &PosTable,
        current_anchor_index: &AnchorIndex,
        extension_cache: &mut Vec<SemiGlobalExtension>,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
    ) -> bool {
        if self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].checked_to_invalid {
            return false
        }
        if self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].left_extension_index.is_none() {
            // Scaled penalty delta of right
            // Always have right extension
            let scaled_penalty_delta_of_right = match self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize].right_extension_index.as_ref().unwrap() {
                SemiGlobalExtensionIndex::Owned(extension_index) => {
                    let extension = &extension_cache[*extension_index as usize].0;
                    (extension.length * cutoff.maximum_scaled_penalty_per_length) as i64 - (extension.penalty * PREC_SCALE) as i64
                },
                SemiGlobalExtensionIndex::Traversed(extension_index, traversed_anchor_index) => {
                    let traversed_anchor = &extension_cache[*extension_index as usize].1[*traversed_anchor_index as usize];
                    (traversed_anchor.remained_length * cutoff.maximum_scaled_penalty_per_length) as i64 - (traversed_anchor.remained_penalty * PREC_SCALE) as i64
                },
            };
            // Generate right extension
            let (optional_left_extension, left_traversed_anchor) = pos_table.extend_left(
                &current_anchor_index,
                pattern_size,
                target,
                query,
                penalties,
                cutoff,
                scaled_penalty_delta_of_right,
                wave_front,
            );
            // Add right extension to extension cache
            match optional_left_extension {
                // If successfully right extended
                Some(left_extension) => {
                    let extension_index = extension_cache.len();
                    left_traversed_anchor.iter().enumerate().for_each(|(traversed_anchor_index, traversed_anchor)| {
                        let anchor_index = &traversed_anchor.anchor_index;
                        let mut anchor = &mut self.0[anchor_index.0 as usize][anchor_index.1 as usize];
                        anchor.left_extension_index = Some(SemiGlobalExtensionIndex::Traversed(extension_index as u32, traversed_anchor_index as u32));
                    });
                    extension_cache.push(SemiGlobalExtension(left_extension, left_traversed_anchor));
                    let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    current_anchor.left_extension_index = Some(SemiGlobalExtensionIndex::Owned(extension_index as u32));
                },
                // If left extension is failed
                None => {
                    let current_anchor = &mut self.0[current_anchor_index.0 as usize][current_anchor_index.1 as usize];
                    current_anchor.registered = true;
                    current_anchor.checked_to_invalid = true;
                    return false
                },
            }
        }
        true
    }
}

#[derive(Debug, Clone)]
struct Anchor {
    // Extension indices
    left_extension_index: Option<SemiGlobalExtensionIndex>,
    right_extension_index: Option<SemiGlobalExtensionIndex>,
    // Failed traversed
    right_minimum_penalty: Option<u32>,
    // Fields for registration
    registered: bool, // If registered in semi-global alignment
    checked_to_invalid: bool, // If it's certain that the alignment cannot satisfy the cutoff
    included: bool, // If included in used symbol
}

impl Anchor {
    fn new_empty() -> Self {
        Self {
            left_extension_index: None,
            right_extension_index: None,
            right_minimum_penalty: None,
            registered: false,
            checked_to_invalid: false,
            included: false,
        }
    }
}

#[derive(Debug, Clone)]
struct SemiGlobalExtension(Extension, Vec<TraversedAnchorDep>);

#[derive(Debug, Clone)]
enum SemiGlobalExtensionIndex {
    Owned(u32), // Extension index
    Traversed(u32, u32), // Extension index, Traversed anchor index
}
impl SemiGlobalExtensionIndex {
    fn side_traversed_anchors<'a>(&self, extension_cache: &'a Vec<SemiGlobalExtension>) -> &'a [TraversedAnchorDep] {
        match self {
            Self::Owned(extension_index) => {
                &extension_cache[*extension_index as usize].1[..]
            },
            Self::Traversed(extension_index, traversed_anchor_index) => {
                &extension_cache[*extension_index as usize].1[..*traversed_anchor_index as usize]
            },
        }
    }
    fn penalty_and_length(&self, extension_cache: &Vec<SemiGlobalExtension>) -> (u32, u32) {
        match self {
            Self::Owned(extension_index) => {
                let extension = &extension_cache[*extension_index as usize].0;
                (extension.penalty, extension.length)
            },
            Self::Traversed(extension_index, traversed_anchor_index) => {
                let traversed_anchor = &extension_cache[*extension_index as usize].1[*traversed_anchor_index as usize];
                (traversed_anchor.remained_penalty, traversed_anchor.remained_length)
            },
        }
    }
    fn valid_anchor_alignment_operations_and_position(
        left: Self,
        right: Self,
        pos_table: &PosTable,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        extension_cache: &Vec<SemiGlobalExtension>,
        cutoff: &Cutoff,
    ) -> Option<(
        u32, // penalty
        u32, // length
        Vec<AlignmentOperations>, // operations
        AlignmentPosition, // position
    )> {
        let anchor_position = &pos_table.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let anchor_size = anchor_position.pattern_count * pattern_size;

        let (left_penalty, left_length) = left.penalty_and_length(&extension_cache);
        let (right_penalty, right_length) = right.penalty_and_length(&extension_cache);

        let penalty = left_penalty + right_penalty;
        let length = left_length + right_length + anchor_size;

        if (
            length >= cutoff.minimum_aligned_length
        ) && (
            (cutoff.maximum_scaled_penalty_per_length * length) >= (penalty * PREC_SCALE)
        ) {
            let anchor_query_position = anchor_index.0 * pattern_size;
            let anchor_record_position = anchor_position.target_position;

            let (left_alignment_operations, left_insertion_count, left_deletion_count) = left.left_operations_and_indel_count(extension_cache);
            let (right_alignment_operations, right_insertion_count, right_deletion_count) = right.right_operations_and_indel_count(extension_cache);

            let alignment_position = AlignmentPosition {
                target: (
                    anchor_record_position + left_deletion_count - left_length,
                    anchor_record_position + anchor_size + right_length - right_deletion_count,
                ),
                query: (
                    anchor_query_position + left_insertion_count - left_length,
                    anchor_query_position + anchor_size + right_length - right_insertion_count,
                ),
            };
            let alignment_operations = AlignmentOperations::concatenate_operations_consuming(
            left_alignment_operations,
            right_alignment_operations,
            anchor_size as u32,
            );

            Some((
                penalty,
                length,
                alignment_operations,
                alignment_position,
            ))
        } else {
            None
        }
    }
    fn right_operations_and_indel_count(&self, extension_cache: &Vec<SemiGlobalExtension>) -> (Vec<AlignmentOperations>, u32, u32) {
        match self {
            SemiGlobalExtensionIndex::Owned(extension_index) => {
                let extension = &extension_cache[*extension_index as usize].0;
                (extension.reversed_operations.clone(), extension.insertion_count, extension.deletion_count)
            },
            SemiGlobalExtensionIndex::Traversed(extension_index, traversed_anchor_index) => {
                let original_extension = &extension_cache[*extension_index as usize].0;
                let traversed_anchor = &extension_cache[*extension_index as usize].1[*traversed_anchor_index as usize];

                let mut operations = original_extension.reversed_operations[traversed_anchor.index_of_operation as usize..].to_vec();
                operations[0].count = traversed_anchor.alternative_match_count;

                let mut insertion_count = 0;
                let mut deletion_count = 0;

                operations.iter().for_each(|alignment_operation| {
                    match alignment_operation.operation {
                        AlignmentOperation::Insertion => {
                            insertion_count += alignment_operation.count;
                        },
                        AlignmentOperation::Deletion => {
                            deletion_count += alignment_operation.count;
                        },
                        _ => {
                            //
                        },
                    }
                });
                
                (operations, insertion_count, deletion_count)
            },
        }
    }
    fn left_operations_and_indel_count(&self, extension_cache: &Vec<SemiGlobalExtension>) -> (Vec<AlignmentOperations>, u32, u32) {
        match self {
            SemiGlobalExtensionIndex::Owned(extension_index) => {
                let extension = &extension_cache[*extension_index as usize].0;
                (extension.reversed_operations.clone(), extension.insertion_count, extension.deletion_count)
            },
            SemiGlobalExtensionIndex::Traversed(extension_index, traversed_anchor_index) => {
                let original_extension = &extension_cache[*extension_index as usize].0;
                let traversed_anchor = &extension_cache[*extension_index as usize].1[*traversed_anchor_index as usize];

                let mut operations = original_extension.reversed_operations[..=traversed_anchor.index_of_operation as usize].to_vec();
                operations.last_mut().unwrap().count = traversed_anchor.alternative_match_count;

                let mut insertion_count = 0;
                let mut deletion_count = 0;

                operations.iter().for_each(|alignment_operation| {
                    match alignment_operation.operation {
                        AlignmentOperation::Insertion => {
                            insertion_count += alignment_operation.count;
                        },
                        AlignmentOperation::Deletion => {
                            deletion_count += alignment_operation.count;
                        },
                        _ => {
                            //
                        },
                    }
                });
                
                (operations, insertion_count, deletion_count)
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemiGlobalAlignment {
    // Symbol
    symbol: Vec<AnchorIndex>, // sorted anchor indices
    // Length and penalty
    penalty: u32,
    length: u32,
    // About operation
    alignment_operations: Vec<AlignmentOperations>,
    alignment_position: AlignmentPosition,
    // About Optimum
    non_optimal_anchor_indices: Vec<AnchorIndex>,
}

impl PosTable {
    pub fn extend_right(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_left: i64,
        wave_front: &mut WaveFront,
    ) -> (Option<Extension>, Vec<TraversedAnchorDep>) {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let right_record_start_index = anchor_position.target_position + anchor_size;
        let right_query_start_index = anchor_index.0 * pattern_size + anchor_size;

        // 
        // (2) Get right extension
        //
        let right_record_slice = &target[right_record_start_index as usize..];
        let right_query_slice = &query[right_query_start_index as usize..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_left, anchor_size, right_query_slice.len() as u32, right_record_slice.len() as u32, penalties, cutoff);

        wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        
        match wave_front.end_point.k {
            Some(last_k) => {
                let last_penalty = wave_front.end_point.penalty;
                let comp_index = (wave_front.wave_front_scores[last_penalty].max_k + last_k) as usize;

                let (right_extension, right_traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(last_penalty as u32, comp_index as u32, penalties, pattern_size);
                let right_traversed_anchors = self.right_traversed_anchors(
                    right_traversed_positions,
                    anchor_index.0,
                    pattern_count,
                    right_record_start_index,
                    right_extension.length,
                    right_extension.penalty,
                    pattern_size,
                );

                (Some(right_extension), right_traversed_anchors)
            },
            None => {
                let last_penalty = wave_front.end_point.penalty;
                let comp_index = wave_front.wave_front_scores[last_penalty].component_index_of_max_length();

                let (right_extension, right_traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(last_penalty as u32, comp_index as u32, penalties, pattern_size);
                let right_traversed_anchors = self.right_traversed_anchors(
                    right_traversed_positions,
                    anchor_index.0,
                    pattern_count,
                    right_record_start_index,
                    right_extension.length,
                    right_extension.penalty,
                    pattern_size,
                );

                (None, right_traversed_anchors)
            },
        }
    }
    pub fn extend_left(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: u32,
        target: &[u8],
        query: &[u8],
        penalties: &Penalty,
        cutoff: &Cutoff,
        scaled_penalty_delta_of_right: i64,
        wave_front: &mut WaveFront,
    ) -> (Option<Extension>, Vec<TraversedAnchorDep>) {
        let anchor_position = &self.0[anchor_index.0 as usize][anchor_index.1 as usize];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.target_position;
        let left_query_last_index = anchor_index.0 * pattern_size;

        // 
        // (2) Get left extension
        //
        let left_record_slice = &target[..left_record_last_index as usize];
        let left_query_slice = &query[..left_query_last_index as usize];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_delta_of_right, anchor_size, left_query_slice.len() as u32, left_record_slice.len() as u32, penalties, cutoff);

        wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        
        match wave_front.end_point.k {
            Some(last_k) => {
                let last_penalty = wave_front.end_point.penalty;
                let comp_index = (wave_front.wave_front_scores[last_penalty].max_k + last_k) as usize;

                let (left_extension, left_traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(last_penalty as u32, comp_index as u32, penalties, pattern_size);
                let left_traversed_anchors = self.left_traversed_anchors(
                    left_traversed_positions,
                    anchor_index.0,
                    left_record_last_index,
                    left_extension.length,
                    left_extension.penalty,
                    pattern_size,
                );

                (Some(left_extension), left_traversed_anchors)
            },
            None => {
                let last_penalty = wave_front.end_point.penalty;
                let comp_index = wave_front.wave_front_scores[last_penalty].component_index_of_max_length();

                let (left_extension, left_traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(last_penalty as u32, comp_index as u32, penalties, pattern_size);
                let left_traversed_anchors = self.left_traversed_anchors(
                    left_traversed_positions,
                    anchor_index.0,
                    left_record_last_index,
                    left_extension.length,
                    left_extension.penalty,
                    pattern_size,
                );

                (None, left_traversed_anchors)
            },
        }
    }
}

impl WaveFrontScore {
    fn component_index_of_max_length(&self) -> usize {
        let mut max_length = 0;
        let mut component_index_of_max_length = 0;

        self.components_by_k.iter().enumerate().for_each(|(component_index, components)| {
            let m_comp = &components.m;
            if m_comp.bt != BackTraceMarker::Empty {
                let length = m_comp.fr + m_comp.deletion_count as i32;
                if max_length < length {
                    max_length = length;
                    component_index_of_max_length = component_index;
                }
            }
        });

        component_index_of_max_length
    }
}
