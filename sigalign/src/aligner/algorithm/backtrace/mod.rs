use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, Extension};
use super::{WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

mod backtrace_wave_front;

#[derive(Debug, Clone)]
pub struct TraversedPosition {
    pub pattern_count_from_start_point: usize,
    pub traversed_record_length_to_anchor: usize,
    pub traversed_length_to_anchor_end: usize,
    pub traversed_penalty_to_anchor_end: usize,
    pub index_of_operation: usize,
    pub alternative_match_count: usize,
}

impl TraversedPosition {
    fn to_right_traversed_anchor(
        self,
        anchor_index: AnchorIndex,
        length_of_extension: usize,
        penalty_of_extension: usize,
        anchor_size: usize,
    ) -> TraversedAnchor {
        TraversedAnchor {
            anchor_index,
            remained_length: length_of_extension - self.traversed_length_to_anchor_end,
            remained_penalty: penalty_of_extension - self.traversed_penalty_to_anchor_end,
            index_of_operation: self.index_of_operation,
            alternative_match_count: self.alternative_match_count,
        }
    }
    fn to_left_traversed_anchor(
        self,
        anchor_index: AnchorIndex,
        length_of_extension: usize,
        penalty_of_extension: usize,
        anchor_size: usize,
    ) -> TraversedAnchor {
        TraversedAnchor {
            anchor_index,
            remained_length: length_of_extension - self.traversed_length_to_anchor_end,
            remained_penalty: penalty_of_extension - self.traversed_penalty_to_anchor_end,
            index_of_operation: self.index_of_operation,
            alternative_match_count: self.alternative_match_count,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraversedAnchor {
    pub anchor_index: AnchorIndex,
    pub remained_length: usize,
    pub remained_penalty: usize,
    pub index_of_operation: usize,
    pub alternative_match_count: usize,
}

impl PosTable {
    pub fn right_traversed_anchors(
        &self,
        traversed_positions: Vec<TraversedPosition>,
        anchor_pattern_index: usize,
        anchor_pattern_count: usize,
        record_start_index: usize,
        length_of_extension: usize,
        penalty_of_extension: usize,
        pattern_size: usize,
    ) -> Vec<TraversedAnchor> {
        traversed_positions.into_iter().map(|traversed_position| {
            let pattern_index = anchor_pattern_index + anchor_pattern_count + traversed_position.pattern_count_from_start_point;
            let pattern_position = &self.0[pattern_index];
            
            let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, record_start_index + traversed_position.traversed_record_length_to_anchor);
            let anchor_position = &pattern_position[anchor_index_in_pattern];

            let traversed_anchor = traversed_position.to_right_traversed_anchor(
                (pattern_index, anchor_index_in_pattern),
                length_of_extension,
                penalty_of_extension,
                anchor_position.pattern_count * pattern_size,
            );
            traversed_anchor
        }).collect()
    }
    pub fn left_traversed_anchors(
        &self,
        traversed_positions: Vec<TraversedPosition>,
        anchor_pattern_index: usize,
        record_last_index: usize,
        length_of_extension: usize,
        penalty_of_extension: usize,
        pattern_size: usize,
    ) -> Vec<TraversedAnchor> {
        traversed_positions.into_iter().map(|traversed_position| {
            let pattern_index = anchor_pattern_index - traversed_position.pattern_count_from_start_point;
            let pattern_position = &self.0[pattern_index];
            
            let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, record_last_index - traversed_position.traversed_record_length_to_anchor);
            let anchor_position = &pattern_position[anchor_index_in_pattern];

            let traversed_anchor = traversed_position.to_left_traversed_anchor(
                (pattern_index, anchor_index_in_pattern),
                length_of_extension,
                penalty_of_extension,
                anchor_position.pattern_count * pattern_size,
            );
            traversed_anchor
        }).collect()
    }
}

impl AnchorPosition {
    fn binary_search_index(pattern_position: &Vec<Self>, record_position: usize) -> usize {
        pattern_position.binary_search_by_key(&record_position, |anchor_position| {
            anchor_position.record_position
        }).unwrap()
    }
}
