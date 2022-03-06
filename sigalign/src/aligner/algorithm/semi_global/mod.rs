use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

fn semi_alignment_query_to_record(
    pos_table: &PosTable,
    left_penalty_margin_for_new_pattern: &Vec<i64>,
    pattern_size: usize,
    record_sequence: Sequence,
    query_sequence: Sequence,
    penalties: &Penalties,
    cutoff: &Cutoff,
    wave_front: &mut WaveFront,
) {
    let sorted_anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
        (0..pattern_position.len()).map(move |anchor_index| {
            (pattern_index, anchor_index)
        })
    }).flatten().collect();
    
    let mut anchor_table: Vec<Vec<Anchor>> = pos_table.0.iter().map(|pattern_position| {
        vec![Anchor::new_empty(); pattern_position.len()]
    }).collect();

    //
    // (1) Get right extension for all anchors
    // Change anchors to 3 cases
    //  - Have owned right extension
    //  - Have traversed right extension
    //  - Have right minimum penalty (when traversed from failed extension)
    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor_need_right_extension = {
            let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
            (
                current_anchor.right_extension_cache.is_none()
            ) && (
                current_anchor.right_minimum_penalty == 0
            )
        };
        if current_anchor_need_right_extension {
            let scaled_left_penalty_margin = left_penalty_margin_for_new_pattern[current_anchor_index.0];
            let (optional_right_extension, right_traversed_anchors) = pos_table.extend_right(
                current_anchor_index,
                pattern_size,
                record_sequence,
                query_sequence,
                penalties,
                cutoff,
                scaled_left_penalty_margin,
                wave_front,
            );

            match optional_right_extension {
                Some(extension) => {
                    // Add right extension to traversed anchors
                    right_traversed_anchors.iter().enumerate().for_each(|(index_of_traversed_anchors, traversed_anchor)| {
                        let traversed_anchor_index = &traversed_anchor.anchor_index;
                        let right_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].right_extension_cache;
                        *right_extension_cache = Some(
                            SemiGlobalExtension::Traversed(ExtensionReference {
                                penalty: traversed_anchor.remained_penalty,
                                length: traversed_anchor.remained_length,
                                anchor_index: current_anchor_index.clone(),
                                index_of_traversed_anchors,
                            })
                        );
                    });

                    // Add right extension to current anchor
                    anchor_table[current_anchor_index.0][current_anchor_index.1].right_extension_cache = Some(
                        SemiGlobalExtension::Owned(extension, right_traversed_anchors)
                    );
                },
                None => {
                    // Add right extension to traversed anchors
                    right_traversed_anchors.iter().for_each(|traversed_anchor| {
                        let traversed_anchor_index = &traversed_anchor.anchor_index;
                        let anchor = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1];
                        let right_remained_penalty = traversed_anchor.remained_penalty;

                        if anchor.right_minimum_penalty < right_remained_penalty {
                            anchor.right_minimum_penalty = right_remained_penalty;
                        };
                    });

                    // Mark current anchor as registered and invalid
                    let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];
                    current_anchor.registered = true;
                    current_anchor.checked_to_invalid = true;
                },
            }
        }
    });

    //
    // (2) Get left extension for all anchors
    // Change anchors to 3 cases
    //  - Have both right and left extensions
    //  - Have only right extension
    sorted_anchor_indices.iter().rev().for_each(|current_anchor_index| {
        let current_anchor_need_left_extension = {
            let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
            !current_anchor.registered && current_anchor.left_extension_cache.is_none()
        };

        if current_anchor_need_left_extension {
            let scaled_right_penalty_margin = {
                let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
                let optional_right_extension = current_anchor.right_extension_cache.as_ref();
                match optional_right_extension {
                    Some(SemiGlobalExtension::Owned(extension, _)) => {
                        (extension.length * cutoff.maximum_penalty_per_scale) as i64 - (extension.penalty * PRECISION_SCALE) as i64
                    },
                    Some(SemiGlobalExtension::Traversed(extension_reference)) => {
                        (extension_reference.length * cutoff.maximum_penalty_per_scale) as i64 - (extension_reference.penalty * PRECISION_SCALE) as i64
                    },
                    None => {
                        let right_minimum_penalty = current_anchor.right_minimum_penalty;

                        let anchor_position = &pos_table.0[current_anchor_index.0][current_anchor_index.1];
                        let pattern_count = anchor_position.pattern_count;
                        let anchor_size = pattern_count * pattern_size;
    
                        let left_record_end_index = anchor_position.record_position;
                        let left_query_end_index = current_anchor_index.0 * pattern_size;

                        let then_right_minimum_length = (record_sequence.len() - left_record_end_index).min(
                            query_sequence.len() - left_query_end_index
                        ) - anchor_size;
    
                        let max_gap = (right_minimum_penalty - penalties.o) / penalties.e;
                        let then_right_maximum_length = then_right_minimum_length + max_gap;
                        
                        (then_right_maximum_length * cutoff.maximum_penalty_per_scale) as i64 - (right_minimum_penalty * PRECISION_SCALE) as i64
                    },
                }
            };
            let (optional_left_extension, left_traversed_anchors) = pos_table.extend_left(
                current_anchor_index,
                pattern_size,
                record_sequence,
                query_sequence,
                penalties,
                cutoff,
                scaled_right_penalty_margin,
                wave_front,
            );

            match optional_left_extension {
                Some(extension) => {
                    let mut leftmost_is_invalid = false;

                    // Add left extension to traversed anchors
                    left_traversed_anchors.iter().enumerate().for_each(|(index_of_traversed_anchors, traversed_anchor)| {
                        let traversed_anchor_index = traversed_anchor.anchor_index.clone();
                        let anchor = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1];
                        
                        if leftmost_is_invalid {
                            anchor.registered = true;
                            anchor.checked_to_invalid = true;
                        } else {
                            if anchor.checked_to_invalid {
                                leftmost_is_invalid = true;
                            } else {
                                let left_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].left_extension_cache;
                                *left_extension_cache = Some(
                                    SemiGlobalExtension::Traversed(ExtensionReference {
                                        penalty: traversed_anchor.remained_penalty,
                                        length: traversed_anchor.remained_length,
                                        anchor_index: current_anchor_index.clone(),
                                        index_of_traversed_anchors,
                                    })
                                );
                            }
                        }
                    });

                    // Add left extension to current anchor
                    let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];
                    if leftmost_is_invalid {
                        current_anchor.registered = true;
                        current_anchor.checked_to_invalid = true;
                    } else {
                        current_anchor.left_extension_cache = Some(
                            SemiGlobalExtension::Owned(extension, left_traversed_anchors)
                        );
                    }
                },
                None => {
                    // Add left extension to traversed anchors
                    left_traversed_anchors.iter().for_each(|traversed_anchor| {
                        let traversed_anchor_index = &traversed_anchor.anchor_index;

                        let anchor = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1];
                        if anchor.right_minimum_penalty != 0 {
                            let minimum_penalty = anchor.right_minimum_penalty + traversed_anchor.remained_penalty;

                            let anchor_position = &pos_table.0[current_anchor_index.0][current_anchor_index.1];
        
                            let left_record_end_index = anchor_position.record_position;
                            let left_query_end_index = current_anchor_index.0 * pattern_size;
        
                            let max_gap = (minimum_penalty - penalties.o) / penalties.e;

                            let then_maximum_length = (
                                left_record_end_index.min(left_query_end_index)
                            )+ (
                                (record_sequence.len() - left_record_end_index).min(query_sequence.len() - left_query_end_index)
                            ) + max_gap;

                            if (minimum_penalty * PRECISION_SCALE) > (then_maximum_length * cutoff.maximum_penalty_per_scale) {
                                anchor.registered = true;
                                anchor.checked_to_invalid = true;
                            }
                        }
                    });

                    // Mark current anchor as registered and invalid
                    let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];
                    current_anchor.registered = true;
                    current_anchor.checked_to_invalid = true;
                },
            }
        }
    });

    // (3) Get semi-global alignments
    // let mut semi_global_alignments: Vec<SemiGlobalAlignment> = Vec::new();
    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];
        if !current_anchor.registered {
            let left_extension_cache = current_anchor.left_extension_cache.as_ref().unwrap();

            //
            // (1) Make right extension
            //
            if current_anchor.right_extension_cache.is_none() {
                let scaled_left_penalty_margin = {
                    match left_extension_cache {
                        SemiGlobalExtension::Owned(extension, _) => {
                            (extension.length * cutoff.maximum_penalty_per_scale) as i64 - (extension.penalty * PRECISION_SCALE) as i64
                        },
                        SemiGlobalExtension::Traversed(extension_reference) => {
                            (extension_reference.length * cutoff.maximum_penalty_per_scale) as i64 - (extension_reference.penalty * PRECISION_SCALE) as i64
                        },
                    }
                };
                let (optional_right_extension, right_traversed_anchors) = pos_table.extend_right(
                    current_anchor_index,
                    pattern_size,
                    record_sequence,
                    query_sequence,
                    penalties,
                    cutoff,
                    scaled_left_penalty_margin,
                    wave_front,
                );
    
                match optional_right_extension {
                    Some(extension) => {
                        let mut rightmost_is_invalid = false;
    
                        // Add right extension to traversed anchors
                        right_traversed_anchors.iter().enumerate().rev().for_each(|(index_of_traversed_anchors, traversed_anchor)| {
                            let traversed_anchor_index = traversed_anchor.anchor_index.clone();
                            let anchor = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1];
                            
                            if rightmost_is_invalid {
                                anchor.registered = true;
                                anchor.checked_to_invalid = true;
                            } else {
                                if anchor.checked_to_invalid {
                                    rightmost_is_invalid = true;
                                } else {
                                    let right_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].right_extension_cache;
                                    if right_extension_cache.is_none() {
                                        *right_extension_cache = Some(
                                            SemiGlobalExtension::Traversed(ExtensionReference {
                                                penalty: traversed_anchor.remained_penalty,
                                                length: traversed_anchor.remained_length,
                                                anchor_index: current_anchor_index.clone(),
                                                index_of_traversed_anchors,
                                            })
                                        );
                                    }
                                }
                            }
                        });
    
                        // Add right extension to current anchor
                        if rightmost_is_invalid {
                            current_anchor.registered = true;
                            current_anchor.checked_to_invalid = true;
                        } else {
                            current_anchor.right_extension_cache = Some(
                                SemiGlobalExtension::Owned(extension, right_traversed_anchors)
                            );
                        }
                    },
                    None => {
                        // Mark current anchor as registered and invalid
                        current_anchor.registered = true;
                        current_anchor.checked_to_invalid = true;
                    },
                }
            }

            if !current_anchor.registered { // Right extension can be none if previous part (1) has been failed.
                //
                // (2) Get penalty and length
                //
                let mut penalty = 0;
                let mut length = 0;

                let right_extension_cache = current_anchor.left_extension_cache.as_ref().unwrap();

                match left_extension_cache {
                    SemiGlobalExtension::Owned(extension, _) => {
                        penalty += extension.penalty;
                        length += extension.length;
                    },
                    SemiGlobalExtension::Traversed(extension_reference) => {
                        penalty += extension_reference.penalty;
                        length += extension_reference.length;
                    },
                }
                match right_extension_cache {
                    SemiGlobalExtension::Owned(extension, _) => {
                        penalty += extension.penalty;
                        length += extension.length;
                    },
                    SemiGlobalExtension::Traversed(extension_reference) => {
                        penalty += extension_reference.penalty;
                        length += extension_reference.length;
                    },
                }

                let anchor_size = pos_table.0[current_anchor_index.0][current_anchor_index.1].pattern_count * pattern_size;
                length += anchor_size;

                if (penalty * PRECISION_SCALE) <= (length * cutoff.maximum_penalty_per_scale) {
                    //
                    // (3) Make semi-global alignment
                    //
                    let mut symbol = Vec::new();

                    let left_alignment_operations = match left_extension_cache {
                        SemiGlobalExtension::Owned(extension, traversed_anchors) => {
                            traversed_anchors.iter().for_each(|traversed_anchor| {
                                symbol.push(traversed_anchor.anchor_index.clone());
                            });
                            extension.operations.clone()
                        },
                        SemiGlobalExtension::Traversed(extension_reference) => {
                            let original_anchor_index = &extension_reference.anchor_index;
                            
                            let original_anchor = &anchor_table[original_anchor_index.0][original_anchor_index.1];
                            
                            if let SemiGlobalExtension::Owned(extension, traversed_anchors) = original_anchor.left_extension_cache.as_ref().unwrap() {
                                let traversed_anchor = &traversed_anchors[extension_reference.index_of_traversed_anchors];
                                let mut operations = extension.operations[traversed_anchor.index_of_operation..].to_vec();
                                operations[0].count = traversed_anchor.alternative_match_count;
                                operations
                            } else {
                                panic!(""); // Safe
                            }
                        },
                    };
                    
                    // leftmost is always me

                    // let semi_global_alignment = SemiGlobalAlignment {
                    //     symbol: Vec<AnchorIndex>, // sorted anchor indices
                    //     // Length and penalty
                    //     penalty: usize,
                    //     length: usize,
                    //     // About operation
                    //     representative_symbol_index: usize,
                    //     valid_anchor_alignment_operations_and_position: Option<(Vec<AlignmentOperation>, AlignmentPosition)>,
                    //     // About Optimum
                    //     leftmost_optimal_symbol_index: usize,
                    //     rightmost_optimal_symbol_index: usize,
                    //     non_optimal_anchor_indices: Vec<AnchorIndex>,
                    // };
                }
            }
        }
    });
}

fn left_penalty_margin_for_new_pattern(
    pattern_count: usize,
    pattern_size: usize,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    cutoff: &Cutoff,
) {
    let left_penalty_margin_for_new_pattern: Vec<i64> = (0..pattern_count).map(|left_pattern_count| {
        let mut min_penalty = (left_pattern_count / 2) * (min_penalty_for_pattern.odd + min_penalty_for_pattern.even);
        min_penalty += (left_pattern_count % 2) * min_penalty_for_pattern.odd;
        let then_max_length =  pattern_size * left_pattern_count + left_pattern_count;
        (then_max_length * cutoff.maximum_penalty_per_scale - min_penalty) as i64
    }).collect();
}

struct AnchorTable(Vec<Vec<Anchor>>);

#[derive(Debug, Clone)]
struct Anchor {
    // Extension cache
    left_extension_cache: Option<SemiGlobalExtension>,
    right_extension_cache: Option<SemiGlobalExtension>,
    // Fields for failed traversed
    right_minimum_penalty: usize, // Record remained penalty if traversed from failed right extension. 0 if not traversed failed.
    // Fields for registration
    registered: bool, // If registered in semi-global alignment
    checked_to_invalid: bool, // If it's certain that the alignment cannot satisfy the cutoff
    included: bool, // If included in used symbol
}

impl Anchor {
    fn new_empty() -> Self {
        Self {
            left_extension_cache: None,
            right_extension_cache: None,
            right_minimum_penalty: 0,
            registered: false,
            checked_to_invalid: false,
            included: false,
        }
    }
}

#[derive(Debug, Clone)]
enum SemiGlobalExtension {
    Owned(Extension, Vec<TraversedAnchor>),
    Traversed(ExtensionReference),
}

#[derive(Debug, Clone)]
struct ExtensionReference {
    penalty: usize,
    length: usize,
    anchor_index: AnchorIndex,
    index_of_traversed_anchors: usize,
}

#[derive(Debug, Clone)]
pub struct SemiGlobalAlignment {
    // Symbol
    symbol: Vec<AnchorIndex>, // sorted anchor indices
    // Length and penalty
    penalty: usize,
    length: usize,
    // About operation
    representative_symbol_index: usize,
    valid_anchor_alignment_operations_and_position: Option<(Vec<AlignmentOperation>, AlignmentPosition)>,
    // About Optimum
    leftmost_optimal_symbol_index: usize,
    rightmost_optimal_symbol_index: usize,
    non_optimal_anchor_indices: Vec<AnchorIndex>,
}


impl PosTable {
    pub fn extend_right(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_left: i64,
        wave_front: &mut WaveFront,
    ) -> (Option<Extension>, Vec<TraversedAnchor>) {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let right_record_start_index = anchor_position.record_position + anchor_size;
        let right_query_start_index = anchor_index.0 * pattern_size + anchor_size;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_left, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        
        match wave_front.end_point.k {
            Some(last_k) => {
                let last_penalty = wave_front.end_point.score;
                let comp_index = (wave_front.wave_front_scores[last_penalty].max_k + last_k) as usize;

                let (right_extension, right_traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(last_penalty, comp_index, penalties, pattern_size);
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
                let last_penalty = wave_front.end_point.score;
                let comp_index = wave_front.wave_front_scores[last_penalty].component_index_of_max_length();

                let (right_extension, right_traversed_positions) = wave_front.backtrace_from_point_checking_right_traversed(last_penalty, comp_index, penalties, pattern_size);
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
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_right: i64,
        wave_front: &mut WaveFront,
    ) -> (Option<Extension>, Vec<TraversedAnchor>) {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.record_position;
        let left_query_last_index = anchor_index.0 * pattern_size;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &record_sequence[..left_record_last_index];
        let left_query_slice = &query_sequence[..left_query_last_index];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_right, anchor_size, left_query_slice.len(), left_record_slice.len(), penalties, cutoff);

        wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        
        match wave_front.end_point.k {
            Some(last_k) => {
                let last_penalty = wave_front.end_point.score;
                let comp_index = (wave_front.wave_front_scores[last_penalty].max_k + last_k) as usize;

                let (left_extension, left_traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(last_penalty, comp_index, penalties, pattern_size);
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
                let last_penalty = wave_front.end_point.score;
                let comp_index = wave_front.wave_front_scores[last_penalty].component_index_of_max_length();

                let (left_extension, left_traversed_positions) = wave_front.backtrace_from_point_checking_left_traversed(last_penalty, comp_index, penalties, pattern_size);
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
