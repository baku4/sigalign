use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, TraversedPosition, TraversedAnchors, TraversedAnchor};
use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker, calculate_spare_penalty};

struct LocalAlignment {
    sorted_anchor_indices: Vec<AnchorIndex>, // Symbol for alignment
    query_length: usize,
    penalty: usize,
    length: usize,
    representative_anchor_index: AnchorIndex,
    leftmost_optimal_index: usize,
    rightmost_optimal_index: usize,
    non_optimal_anchor_indices: Vec<AnchorIndex>,
}

struct AnchorTable(Vec<Vec<Anchor>>);

#[derive(Debug, Clone)]
struct Anchor {
    extension: Option<LocalExtension>,
    registered_alignment_index: Option<usize>, // For only optimally registered
}
impl Anchor {
    fn new_empty() -> Self {
        Self {
            extension: None,
            registered_alignment_index: None,
        }
    }
}

#[derive(Debug, Clone)]
struct LocalExtension {
    left_extension: Extension,
    right_extension: Extension,
    left_traversed_anchors: Vec<AnchorIndex>,
    right_traversed_anchors: Vec<AnchorIndex>,
}

fn local_alignment(
    pos_table: &PosTable,
    pattern_size: usize,
    record_sequence: Sequence,
    query_sequence: Sequence,
    penalties: &Penalties,
    min_penalty_for_pattern: &MinPenaltyForPattern,
    cutoff: &Cutoff,
    left_wave_front: &mut WaveFront,
    right_wave_front: &mut WaveFront,
) {
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

    let local_alignments: Vec<LocalAlignment> = Vec::new();

    sorted_anchor_indices.into_iter().for_each(|anchor_index| {
        let current_anchor = &mut anchor_table[anchor_index.0][anchor_index.1];
    
        let need_extension = current_anchor.registered_alignment_index.is_none();
        if need_extension {
            //
        } else {
            // need_extension == false
        }
    });
}

impl PosTable {
    fn get_extension(
        &self,
        anchor_index: &AnchorIndex,
        pattern_size: usize,
        record_sequence: Sequence,
        query_sequence: Sequence,
        penalties: &Penalties,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        cutoff: &Cutoff,
        left_wave_front: &mut WaveFront,
        right_wave_front: &mut WaveFront,
    ) {
        let anchor_position = &self.0[anchor_index.0][anchor_index.1];
        let pattern_count = anchor_position.pattern_count;
        let anchor_size = pattern_count * pattern_size;

        // Get right extension
        let right_record_start_index = anchor_position.record_position + anchor_size;
        let right_query_start_index = anchor_index.0 * pattern_size + anchor_size;

        let right_record_slice = &record_sequence[right_record_start_index..];
        let right_query_slice = &query_sequence[right_query_start_index..];

        let scaled_penalty_margin_of_left = ((pattern_size - 1) * cutoff.maximum_penalty_per_scale) as i64; // Assuming this anchor is leftmost of alignment (It is safe)
        let right_spare_penalty = calculate_spare_penalty(scaled_penalty_margin_of_left, anchor_size, right_query_slice.len(), right_record_slice.len(), penalties, cutoff);

        right_wave_front.align_right_to_end_point(right_record_slice, right_query_slice, penalties, right_spare_penalty);
        let right_vpc_vector = right_wave_front.get_sorted_vpc_vector(cutoff.maximum_penalty_per_scale);
    }
}

// Validate Position Candidate
struct VPC {
    query_length: usize,
    penalty_margin: usize,
    penalty: usize,
    component_index: usize,
}

impl VPC {
    fn new_of_anchor_end() -> Self {
        Self {
            query_length: 0,
            penalty_margin: 0,
            penalty: 0,
            component_index: 0,
        }
    }
}

impl WaveFront {
    // Sorted by query length
    // --------------------------------
    // | QL |<QL |<QL |<QL | ... |<QL |
    // | PM>| PM>| PM>| PM>| ... | PM |
    // --------------------------------
    fn get_sorted_vpc_vector(&self, maximum_penalty_per_scale: usize) -> Vec<VPC> {
        let last_score = self.end_point.score;

        let mut sorted_vpc_vector: Vec<VPC> = Vec::new();
        let mut max_query_length = 0;
        let mut max_penalty_margin = 0;
        // TODO: Next to do

        let vpc_vector: Vec<VPC> = self.wave_front_scores[..last_score].iter().enumerate().map(|(penalty, wave_front_score)| {
            let (max_query_length, length, comp_index) = wave_front_score.point_of_maximum_query_length();
            let penalty_margin = length as usize * maximum_penalty_per_scale - penalty;
            let vpc = VPC {
                query_length: max_query_length as usize,
                penalty_margin: penalty_margin,
                penalty: penalty,
                component_index: comp_index,
            };
            vpc
        }).collect();

        vpc_vector
    }
}

impl WaveFrontScore {
    fn point_of_maximum_query_length(&self) -> (i32, i32, usize) { // (Maximum query index, Length of that, Component index of that)
        let mut max_query_length = i32::MIN;
        let mut length_cache = 0;
        let mut comp_index_cache = 0;

        self.components_by_k.iter().enumerate().for_each(|(comp_index, comp)| {
            let query_length = self.max_k - comp.m.fr - comp_index as i32; // Fr + k
            if max_query_length < query_length {
                max_query_length = query_length;
                length_cache = comp.m.fr + comp.m.deletion_count as i32;
                comp_index_cache = comp_index;
            }
        });

        (max_query_length, length_cache, comp_index_cache)
    }
}
