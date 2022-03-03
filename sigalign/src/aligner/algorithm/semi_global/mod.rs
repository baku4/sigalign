use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

type PenaltyMargin = i64;

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

    // let mut semi_global_alignments: Vec<SemiGlobalAlignment> = Vec::new();
    sorted_anchor_indices.into_iter().for_each(|current_anchor_index| {
        let current_anchor = &mut anchor_table[current_anchor_index.0][current_anchor_index.1];

        if !current_anchor.registered {
            //
            // (1) Get right extension of current anchor
            //

            // TODO:
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
    left_extension_cache: Option<SemiGlobalExtension>,
    right_extension_cache: Option<SemiGlobalExtension>,
    registered: bool, // If registered in semi alignment
    checked_to_invalid: bool, // If extension failed
    included: bool, // If included in used symbol
}
impl Anchor {
    fn new_empty() -> Self {
        Self {
            left_extension_cache: None,
            right_extension_cache: None,
            registered: false,
            checked_to_invalid: false,
            included: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SemiGlobalExtension {
    pub valid_extension: Option<Extension>,
    pub traversed_anchors: Vec<TraversedAnchor>,
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
