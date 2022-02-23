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
pub use backtrace_wave_front::{TraversedPositions};

#[derive(Debug, Clone)]
pub struct TraversedAnchors {
    pub sorted_list: Vec<AnchorIndex>,
    pub at_the_end: Option<AnchorIndex>,
}

impl PosTable {
    pub fn check_right_traversed(&self, pattern_start_index: usize, record_start_index: usize, traversed_positions: &TraversedPositions) -> TraversedAnchors {
        let sorted_list: Vec<AnchorIndex> = traversed_positions.iter().enumerate().filter_map(|(pattern_index_from_start, optional_record_slice_index)| {
            match optional_record_slice_index {
                Some(v) => {
                    let pattern_index = pattern_start_index + pattern_index_from_start;
                    let pattern_position = &self.0[pattern_index];
                    let anchor_index = AnchorPosition::binary_search_index(pattern_position, record_start_index + *v);
                    Some((pattern_index, anchor_index))
                },
                None => {
                    None
                },
            }
        }).collect();

        let rightmost_anchor_index = match sorted_list.last() {
            Some(v) => Some(v.clone()),
            None => None,
        };

        TraversedAnchors {
            sorted_list,
            at_the_end: rightmost_anchor_index,
        }
    }
    pub fn check_left_traversed(&self, pattern_last_index: usize, record_last_index: usize, traversed_positions: &TraversedPositions) -> TraversedAnchors {
        let sorted_list: Vec<AnchorIndex> = traversed_positions.iter().enumerate().filter_map(|(pattern_index_from_last, optional_record_slice_index)| {
            match optional_record_slice_index {
                Some(v) => {
                    let pattern_index = pattern_last_index - pattern_index_from_last;
                    let pattern_position = &self.0[pattern_index];
                    let anchor_index = AnchorPosition::binary_search_index(pattern_position, record_last_index - *v);
                    Some((pattern_index, anchor_index))
                },
                None => {
                    None
                },
            }
        }).collect();

        let leftmost_anchor_index = match sorted_list.last() {
            Some(v) => Some(v.clone()),
            None => None,
        };

        TraversedAnchors {
            sorted_list,
            at_the_end: leftmost_anchor_index,
        }
    }
}

impl AnchorPosition {
    fn binary_search_index(pattern_position: &Vec<Self>, record_position: usize) -> usize {
        pattern_position.binary_search_by_key(&record_position, |anchor_position| {
            anchor_position.record_position
        }).unwrap()
    }
}
