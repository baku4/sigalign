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
    //
    sorted_anchor_indices.iter().for_each(|current_anchor_index| {
        let current_anchor_need_right_extension = {
            let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
            current_anchor.right_extension_cache.is_none()
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
                    let mut symbol = Vec::new();
                    right_traversed_anchors.into_iter().rev().for_each(|mut traversed_anchor| { // Reverse for sorted symbol
                        let traversed_anchor_index = traversed_anchor.anchor_index.clone();
                        traversed_anchor.anchor_index = current_anchor_index.clone(); // Change traversed anchor index to current anchor index

                        let right_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].right_extension_cache;
                        *right_extension_cache = Some(
                            SemiGlobalExtensionDep::Traversed(traversed_anchor, symbol.clone())
                        );

                        symbol.push(traversed_anchor_index);
                    });

                    // Add right extension to current anchor
                    anchor_table[current_anchor_index.0][current_anchor_index.1].right_extension_cache = Some(
                        SemiGlobalExtensionDep::Owned(extension, symbol)
                    );
                },
                None => {
                    // Add right extension to traversed anchors
                    right_traversed_anchors.into_iter().for_each(|traversed_anchor| {
                        let traversed_anchor_index = &traversed_anchor.anchor_index;

                        let right_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].right_extension_cache;
                        if right_extension_cache.is_none() {
                            *right_extension_cache = Some(
                                SemiGlobalExtensionDep::FailedTraversed(traversed_anchor.remained_penalty)
                            );
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

    //
    // (2) Get left extension for all anchors
    //
    sorted_anchor_indices.iter().rev().for_each(|current_anchor_index| {
        let current_anchor_need_left_extension = {
            let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
            !current_anchor.registered && current_anchor.left_extension_cache.is_none()
        };

        if current_anchor_need_left_extension {
            let scaled_right_penalty_margin = {
                let current_anchor = &anchor_table[current_anchor_index.0][current_anchor_index.1];
                let right_extension = current_anchor.right_extension_cache.as_ref().unwrap();
                match right_extension {
                    SemiGlobalExtensionDep::Owned(extension, _) => {
                        (extension.length * cutoff.maximum_penalty_per_scale) as i64 - (extension.penalty * PRECISION_SCALE) as i64
                    },
                    SemiGlobalExtensionDep::Traversed(traversed_anchor, _) => {
                        (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64
                    },
                    SemiGlobalExtensionDep::FailedTraversed(penalty) => {
                        let anchor_position = &pos_table.0[current_anchor_index.0][current_anchor_index.1];
                        let pattern_count = anchor_position.pattern_count;
                        let anchor_size = pattern_count * pattern_size;
    
                        let right_record_start_index = anchor_position.record_position + anchor_size;
                        let right_query_start_index = current_anchor_index.0 * pattern_size + anchor_size;
    
                        let right_record_slice_len = record_sequence.len() - right_record_start_index;
                        let right_query_slice_len = query_sequence.len() - right_query_start_index;
    
                        let max_gap = (penalty - penalties.o) / penalties.e;
                        let length = right_record_slice_len.min(right_query_slice_len) + max_gap;
                        
                        (length * cutoff.maximum_penalty_per_scale) as i64 - (penalty * PRECISION_SCALE) as i64
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
                    let mut symbol = Vec::new();
                    left_traversed_anchors.into_iter().for_each(|mut traversed_anchor| {
                        let traversed_anchor_index = traversed_anchor.anchor_index.clone();
                        let anchor = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1];
                        
                        if leftmost_is_invalid {
                            anchor.registered = true;
                            anchor.checked_to_invalid = true;
                        } else {
                            if anchor.checked_to_invalid {
                                leftmost_is_invalid = true;
                            } else {
                                traversed_anchor.anchor_index = current_anchor_index.clone(); // Change traversed anchor index to current anchor index
                                
                                let left_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].left_extension_cache;
                                *left_extension_cache = Some(
                                    SemiGlobalExtensionDep::Traversed(traversed_anchor, symbol.clone())
                                );

                                symbol.push(traversed_anchor_index);
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
                            SemiGlobalExtensionDep::Owned(extension, symbol)
                        );
                    }
                },
                None => {
                    // Add left extension to traversed anchors
                    left_traversed_anchors.into_iter().for_each(|traversed_anchor| {
                        let traversed_anchor_index = &traversed_anchor.anchor_index;

                        let left_extension_cache = &mut anchor_table[traversed_anchor_index.0][traversed_anchor_index.1].left_extension_cache;
                        if left_extension_cache.is_none() {
                            *left_extension_cache = Some(
                                SemiGlobalExtensionDep::FailedTraversed(traversed_anchor.remained_penalty)
                            );
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
        //
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
    left_extension_cache: Option<SemiGlobalExtensionDep>,
    right_extension_cache: Option<SemiGlobalExtensionDep>,
    left_minimum_penalty: Option<usize>,
    right_minimum_penalty: Option<usize>,
    registered: bool, // If registered in semi alignment
    checked_to_invalid: bool, // If it's certain that the alignment cannot satisfy the cutoff
    included: bool, // If included in used symbol
}

impl Anchor {
    fn new_empty() -> Self {
        Self {
            left_extension_cache: None,
            right_extension_cache: None,
            left_minimum_penalty: None,
            right_minimum_penalty: None,
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
    peanlty: usize,
    length: usize,
    anchor_index: AnchorIndex,
    index_of_traversed_anchors: usize,
}

#[derive(Debug, Clone)]
enum SemiGlobalExtensionDep {
    Owned(Extension, Vec<AnchorIndex>), // Extension, Symbol (sorted)
    Traversed(TraversedAnchor, Vec<AnchorIndex>), // TraversedAnchor, Symbol (sorted)
    FailedTraversed(usize), // Min penalty
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
    left_traversed_anchors: Vec<TraversedAnchor>,
    right_traversed_anchors: Vec<TraversedAnchor>,
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
