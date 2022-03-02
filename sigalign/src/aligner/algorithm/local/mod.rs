use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider, pos_table,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

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
    valid_anchor_alignment_operations: Option<Vec<AlignmentOperation>>, // None, if (len < cutoff len)
    left_traversed_anchors: Vec<TraversedAnchor>,
    right_traversed_anchors: Vec<TraversedAnchor>,
    // About Optimum
    leftmost_optimal_symbol_index: usize,
    rightmost_optimal_symbol_index: usize,
    non_optimal_anchor_indices: Vec<AnchorIndex>,
}

struct AnchorTable(Vec<Vec<Anchor>>);

#[derive(Debug, Clone)]
struct Anchor {
    registered: bool,
    extensions_cache: Option<LocalExtension>,
}
impl Anchor {
    fn new_empty() -> Self {
        Self {
            registered: false,
            extensions_cache: None,
        }
    }
}

#[derive(Debug, Clone)]
struct LocalExtension {
    left_extension: Extension,
    right_extension: Extension,
    left_traversed_anchors: Vec<TraversedAnchor>,
    right_traversed_anchors: Vec<TraversedAnchor>,
    left_scaled_penalty_margins: Vec<i64>,
    right_scaled_penalty_margins: Vec<i64>,
}

pub fn local_alignment(
    pos_table: &PosTable,
    pattern_size: usize,
    record_sequence: Sequence,
    query_sequence: Sequence,
    penalties: &Penalties,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) -> Vec<LocalAlignment> {
    let pattern_count = pos_table.0.len();
    let scaled_penalty_margin_for_leftmost = (pattern_size - 1) * cutoff.maximum_penalty_per_scale;

    let sorted_anchor_indices: Vec<AnchorIndex> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
        (0..pattern_position.len()).map(move |anchor_index| {
            (pattern_index, anchor_index)
        })
    }).flatten().collect();

    let mut anchor_table: Vec<Vec<Anchor>> = pos_table.0.iter().enumerate().map(|(pattern_index, pattern_position)| {
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
            let anchor_size = pos_table.0[current_anchor_index.0][current_anchor_index.1].pattern_count * pattern_size;
            let length = left_extension.length + right_extension.length + anchor_size;
            let query_length = length - left_extension.insertion_count as usize - right_extension.insertion_count as usize;
            let penalty = left_extension.penalty + right_extension.penalty;
            let valid_anchor_alignment_operations = if length >= cutoff.minimum_aligned_length {
               let alignment_operations = AlignmentOperation::concatenate_operations(
                   left_extension.operations,
                   right_extension.operations,
                   anchor_size as u32,
                );
                Some(alignment_operations)
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
                valid_anchor_alignment_operations,
                left_traversed_anchors,
                right_traversed_anchors,
                // About Optimum
                leftmost_optimal_symbol_index,
                rightmost_optimal_symbol_index,
                non_optimal_anchor_indices,
            };

            local_alignments.push(local_alignment);
        } else {
            // need_extension == false
        }
    });

    local_alignments
}

impl PosTable {
    fn extend_with_right_first(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_left: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtension {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.record_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_margin = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get right extension & VPC vector
        //
        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_left, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - scaled_penalty_margin_of_left;
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_margin);

        // 
        // (3) Get left extension & VPC vector
        //
        let left_record_slice = &record_sequence[..left_record_last_index];
        let left_query_slice = &query_sequence[..left_query_last_index];

        let right_max_scaled_penalty_margin = right_vpc_vector[0].scaled_penalty_margin as i64;
        let left_spare_penalty = calculate_spare_penalty(right_max_scaled_penalty_margin, anchor_size, left_query_slice.len(), left_record_slice.len(), penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_margin = -anchor_scaled_penalty_margin - right_max_scaled_penalty_margin;
        let left_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_margin);

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = VPC::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_margin, anchor_size);
        // println!("right_spare_penalty: {}", right_spare_penalty);
        // println!("right_wave_front.end_point.score: {}", right_wave_front.end_point.score);
        // println!("right_vpc_vector: {:?}", right_vpc_vector);
        // println!("left_spare_penalty: {}", left_spare_penalty);
        // println!("left_wave_front.end_point.score: {}", left_wave_front.end_point.score);
        // println!("left_vpc_vector: {:?}", left_vpc_vector);

        // println!("left_vpc_index: {:?}", left_vpc_index);
        // println!("right_vpc_index: {:?}", right_vpc_index);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        // println!("left_extension: {:?}", left_extension);
        // println!("left_traversed_positions: {:?}", left_traversed_positions);
        // println!("right_extension: {:?}", right_extension);
        // println!("right_traversed_positions: {:?}", right_traversed_positions);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty margin
        //
        let left_scaled_penalty_margin_of_extension = (left_extension.length * cutoff.maximum_penalty_per_scale) as i64 - (left_extension.penalty * PRECISION_SCALE) as i64;
        let left_max_scaled_penalty_margin = left_vpc_vector[0].scaled_penalty_margin;
        let left_scaled_penalty_margins: Vec<i64> = left_traversed_anchors.iter().map(|traversed_anchor| {
            let remained_scaled_penalty_margin = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64;

            left_max_scaled_penalty_margin + remained_scaled_penalty_margin - left_scaled_penalty_margin_of_extension
        }).collect();

        let right_scaled_penalty_margin_of_extension = (right_extension.length * cutoff.maximum_penalty_per_scale) as i64 - (right_extension.penalty * PRECISION_SCALE) as i64;
        let right_max_scaled_penalty_margin = right_vpc_vector[0].scaled_penalty_margin;
        let right_scaled_penalty_margins: Vec<i64> = right_traversed_anchors.iter().map(|traversed_anchor| {
            let remained_scaled_penalty_margin = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64;

            right_max_scaled_penalty_margin + remained_scaled_penalty_margin - right_scaled_penalty_margin_of_extension
        }).collect();

        LocalExtension {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_margins,
            right_scaled_penalty_margins,
        }
    }
    fn extend_with_left_first(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        cutoff: &Cutoff,
        scaled_penalty_margin_of_right: i64,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) -> LocalExtension {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        //
        // (1) Calculate index
        //
        let left_record_last_index = anchor_position.record_position;
        let right_record_start_index = left_record_last_index + anchor_size;

        let left_query_last_index = anchor_index.0 * pattern_size;
        let right_query_start_index = left_query_last_index + anchor_size;

        let anchor_scaled_penalty_margin = (anchor_size * cutoff.maximum_penalty_per_scale) as i64;

        // 
        // (2) Get left extension & VPC vector
        //
        let left_record_slice = &record_sequence[..left_record_last_index];
        let left_query_slice = &query_sequence[..left_query_last_index];

        let left_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_right, anchor_size, left_query_slice.len(), left_record_slice.len(), penalties, cutoff);

        left_wave_front.align_left_to_end_point(left_record_slice, left_query_slice, penalties, left_spare_penalty);
        let left_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - scaled_penalty_margin_of_right;
        let left_vpc_vector = left_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, left_minimum_scaled_penalty_margin);

        // 
        // (3) Get right extension & VPC vector
        //
        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let left_max_scaled_penalty_margin = left_vpc_vector[0].scaled_penalty_margin as i64;
        let right_spare_penalty = calculate_spare_penalty(left_max_scaled_penalty_margin, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_minimum_scaled_penalty_margin = - anchor_scaled_penalty_margin - left_max_scaled_penalty_margin;
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale, right_minimum_scaled_penalty_margin);

        //
        // (4) Find optimal position of VPC vectors
        //
        let (left_vpc_index, right_vpc_index) = VPC::get_optimal_position(&left_vpc_vector, &right_vpc_vector, anchor_scaled_penalty_margin, anchor_size);
        // println!("right_spare_penalty: {}", right_spare_penalty);
        // println!("right_wave_front.end_point.score: {}", right_wave_front.end_point.score);
        // println!("right_vpc_vector: {:?}", right_vpc_vector);
        // println!("left_spare_penalty: {}", left_spare_penalty);
        // println!("left_wave_front.end_point.score: {}", left_wave_front.end_point.score);
        // println!("left_vpc_vector: {:?}", left_vpc_vector);

        // println!("left_vpc_index: {:?}", left_vpc_index);
        // println!("right_vpc_index: {:?}", right_vpc_index);

        //
        // (5) Get extensions
        //
        let left_vpc = &left_vpc_vector[left_vpc_index];
        let right_vpc = &right_vpc_vector[right_vpc_index];

        let (left_extension, left_traversed_positions) = left_wave_front.backtrace_from_point_checking_left_traversed(left_vpc.penalty, left_vpc.component_index, penalties, pattern_size);
        let (right_extension, right_traversed_positions) = right_wave_front.backtrace_from_point_checking_right_traversed(right_vpc.penalty, right_vpc.component_index, penalties, pattern_size);

        // println!("left_extension: {:?}", left_extension);
        // println!("left_traversed_positions: {:?}", left_traversed_positions);
        // println!("right_extension: {:?}", right_extension);
        // println!("right_traversed_positions: {:?}", right_traversed_positions);

        let left_traversed_anchors = self.left_traversed_anchors(
            left_traversed_positions,
            anchor_index.0,
            left_record_last_index,
            left_extension.length,
            left_extension.penalty,
            pattern_size,
        );
        let right_traversed_anchors = self.right_traversed_anchors(
            right_traversed_positions,
            anchor_index.0,
            pattern_count,
            right_record_start_index,
            right_extension.length,
            right_extension.penalty,
            pattern_size,
        );

        //
        // (6) Scaled penalty margin
        //
        let left_scaled_penalty_margin_of_extension = (left_extension.length * cutoff.maximum_penalty_per_scale) as i64 - (left_extension.penalty * PRECISION_SCALE) as i64;
        let left_max_scaled_penalty_margin = left_vpc_vector[0].scaled_penalty_margin;
        let left_scaled_penalty_margins: Vec<i64> = left_traversed_anchors.iter().map(|traversed_anchor| {
            let remained_scaled_penalty_margin = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64;

            left_max_scaled_penalty_margin + remained_scaled_penalty_margin - left_scaled_penalty_margin_of_extension
        }).collect();

        let right_scaled_penalty_margin_of_extension = (right_extension.length * cutoff.maximum_penalty_per_scale) as i64 - (right_extension.penalty * PRECISION_SCALE) as i64;
        let right_max_scaled_penalty_margin = right_vpc_vector[0].scaled_penalty_margin;
        let right_scaled_penalty_margins: Vec<i64> = right_traversed_anchors.iter().map(|traversed_anchor| {
            let remained_scaled_penalty_margin = (traversed_anchor.remained_length * cutoff.maximum_penalty_per_scale) as i64 - (traversed_anchor.remained_penalty * PRECISION_SCALE) as i64;

            right_max_scaled_penalty_margin + remained_scaled_penalty_margin - right_scaled_penalty_margin_of_extension
        }).collect();

        LocalExtension {
            left_extension,
            right_extension,
            left_traversed_anchors,
            right_traversed_anchors,
            left_scaled_penalty_margins,
            right_scaled_penalty_margins,
        }
    }
}

// Validate Position Candidate
#[derive(Debug, Clone)]
struct VPC {
    query_length: usize,
    penalty: usize,
    component_index: usize,
    scaled_penalty_margin: i64,
}

impl VPC {
    fn new_of_anchor_end() -> Self {
        Self {
            query_length: 0,
            penalty: 0,
            component_index: 0,
            scaled_penalty_margin: 0,
        }
    }
    // Return optimal vpc index of (left, right)
    fn get_optimal_position(
        left_vpc_vector: &Vec<Self>,
        right_vpc_vector: &Vec<Self>,
        anchor_scaled_penalty_margin: i64,
        anchor_size: usize,
    ) -> (usize, usize) {
        let mut optimal_left_vpc_index = 0;
        let mut optimal_right_vpc_index = 0;
        let mut optimal_max_query_length = 0;

        for (left_vpc_index, left_vpc) in left_vpc_vector.iter().enumerate().rev() {
            for (right_vpc_index, right_vpc) in right_vpc_vector.iter().enumerate().rev() {
                let scaled_penalty_margin = left_vpc.scaled_penalty_margin + right_vpc.scaled_penalty_margin + anchor_scaled_penalty_margin;

                if scaled_penalty_margin >= 0 {
                    let query_length = left_vpc.query_length + right_vpc.query_length + anchor_size;
                    if optimal_max_query_length < query_length {
                        optimal_max_query_length = query_length;
                        optimal_left_vpc_index = left_vpc_index;
                        optimal_right_vpc_index = right_vpc_index;
                    }
                    break
                }
            }
        }
        
        (optimal_left_vpc_index, optimal_right_vpc_index)
    }
}

impl WaveFront {
    // Sorted by query length
    // --------------------------------
    // | QL |<QL |<QL |<QL | ... |<QL |
    // | PM>| PM>| PM>| PM>| ... | PM |
    // --------------------------------
    fn get_sorted_vpc_vector(&self, maximum_penalty_per_scale: usize, minimum_scaled_penalty_margin: i64) -> Vec<VPC> {
        let last_score = self.end_point.score;

        let mut sorted_vpc_vector: Vec<VPC> = Vec::new();

        self.wave_front_scores[..=last_score].iter().enumerate().for_each(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let scaled_penalty_margin = (length as usize * maximum_penalty_per_scale) as i64 - (penalty * PRECISION_SCALE) as i64;

            if minimum_scaled_penalty_margin <= scaled_penalty_margin {
                let mut ql_index_to_insert: usize = 0;
                let mut pm_index_to_insert: usize = 0;
                let mut ql_is_same_as_pre = false;

                // Find index to insert
                for (index, vpc_in_vector) in sorted_vpc_vector.iter().enumerate().rev() {
                    // QL
                    if ql_index_to_insert == 0 {
                        let checked_sub = max_query_length.checked_sub(vpc_in_vector.query_length as usize);
                        if let Some(gap) = checked_sub {
                            if gap == 0 {
                                ql_is_same_as_pre = true;
                            }
                            ql_index_to_insert = index + 1;
                        }
                    }
                    // PM
                    if pm_index_to_insert == 0 {
                        if vpc_in_vector.scaled_penalty_margin > scaled_penalty_margin {
                            pm_index_to_insert = index + 1;
                        }
                    }
                    if ql_index_to_insert != 0 && pm_index_to_insert != 0 {
                        break;
                    }
                }

                if ql_index_to_insert > pm_index_to_insert {
                    // Delete middle elements and insert new
                    (0..ql_index_to_insert-pm_index_to_insert).for_each(|_| {
                        sorted_vpc_vector.remove(pm_index_to_insert);
                    });
                    sorted_vpc_vector.insert(
                        pm_index_to_insert,
                        VPC {
                            query_length: max_query_length,
                            scaled_penalty_margin,
                            penalty: penalty,
                            component_index: comp_index,
                        },
                    );
                } else if ql_index_to_insert == pm_index_to_insert {
                    if !ql_is_same_as_pre {
                        if ql_index_to_insert == sorted_vpc_vector.len() {
                            sorted_vpc_vector.insert(
                                pm_index_to_insert,
                                VPC {
                                    query_length: max_query_length,
                                    scaled_penalty_margin,
                                    penalty: penalty,
                                    component_index: comp_index,
                                },
                            );
                        } else {
                            if sorted_vpc_vector[ql_index_to_insert].scaled_penalty_margin < scaled_penalty_margin {
                                sorted_vpc_vector.insert(
                                    pm_index_to_insert,
                                    VPC {
                                        query_length: max_query_length,
                                        scaled_penalty_margin,
                                        penalty: penalty,
                                        component_index: comp_index,
                                    },
                                );
                            }
                        }
                    }
                }
            }
        });
            
        sorted_vpc_vector
    }
}

impl WaveFrontScore {
    fn point_of_maximum_query_length(&self) -> (usize, i32, usize) { // (Maximum query index, Length of that, Component index of that)
        let mut max_query_length = 0;
        let mut length_cache = 0;
        let mut comp_index_cache = 0;

        self.components_by_k.iter().enumerate().for_each(|(comp_index, comp)| {
            if comp.m.bt != BackTraceMarker::Empty {
                let query_length = comp.m.fr + self.max_k - comp_index as i32; // Fr - k
                if max_query_length < query_length {
                    max_query_length = query_length;
                    length_cache = comp.m.fr + comp.m.deletion_count as i32;
                    comp_index_cache = comp_index;
                }
            }
        });

        (max_query_length as usize, length_cache, comp_index_cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone)]
    struct MyStruct {
        ql: usize,
        pm: usize,
    }

    #[test]
    fn print_testing_vpc_with_my_struct() {
        // let mut vector: Vec<MyStruct> = Vec::new();
        let mut vector: Vec<MyStruct> = vec![
            MyStruct { ql: 0, pm: 0 },
        ];

        let my_structs = vec![
            MyStruct { ql: 10, pm: 10 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 8, pm: 12 },
            MyStruct { ql: 30, pm: 60 },
            MyStruct { ql: 3, pm: 4 },
            MyStruct { ql: 14, pm: 20 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 30, pm: 50 },
            MyStruct { ql: 12, pm: 6 },
            MyStruct { ql: 32, pm: 40 },
            MyStruct { ql: 25, pm: 30 },
            MyStruct { ql: 18, pm: 5 },
        ];

        for my_struct in my_structs {
            println!("my_struct: {:?}", my_struct);
            let (ql, pm) = (my_struct.ql, my_struct.pm);

            let mut ql_index_to_insert: usize = 0;
            let mut pm_index_to_insert: usize = 0;
            let mut ql_is_same_as_pre = false;

            // Find index to insert
            for (index, my_struct_in_vector) in vector.iter().enumerate().rev() {
                // QL
                if ql_index_to_insert == 0 {
                    let checked_sub = ql.checked_sub(my_struct_in_vector.ql);
                    if let Some(gap) = checked_sub {
                        if gap == 0 {
                            ql_is_same_as_pre = true;
                        }
                        ql_index_to_insert = index + 1;
                    }
                }
                // PM
                if pm_index_to_insert == 0 {
                    if my_struct_in_vector.pm > pm {
                        pm_index_to_insert = index + 1;
                    }
                }
                if ql_index_to_insert != 0 && pm_index_to_insert != 0 {
                    break;
                }
            }

            println!("{}, {}", ql_index_to_insert, pm_index_to_insert);

            if ql_index_to_insert > pm_index_to_insert {
                // Delete middle elements and insert new
                (0..ql_index_to_insert-pm_index_to_insert).for_each(|_| {
                    vector.remove(pm_index_to_insert);
                });
                vector.insert(pm_index_to_insert, my_struct);
            } else if ql_index_to_insert == pm_index_to_insert {
                if !ql_is_same_as_pre {
                    if ql_index_to_insert == vector.len() {
                        vector.insert(pm_index_to_insert, my_struct);
                    } else {
                        if vector[ql_index_to_insert].pm < pm {
                            vector.insert(pm_index_to_insert, my_struct);
                        }
                    }
                }
            }

            println!("{:#?}", vector);
        }

        println!("{:#?}", vector);
    }
}
