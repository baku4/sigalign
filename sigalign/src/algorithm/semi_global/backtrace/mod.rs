use crate::core::SeqLen;

use super::{PosTable, AnchorPosition, AnchorIndex, Extension};
use super::{WaveFront, BackTraceMarker};

mod backtrace_wave_front;

#[derive(Debug, Clone)]
pub struct TraversedPosition {
    pub pattern_count_from_start_point: u32,
    pub traversed_record_length_to_anchor: u32,
    pub traversed_length_to_anchor_end: u32,
    pub traversed_penalty_to_anchor_end: u32,
    pub index_of_operation: u32,
    pub alternative_match_count: u32,
}

#[derive(Debug, Clone)]
pub struct TraversedAnchorDep {
    pub anchor_index: AnchorIndex,
    pub remained_length: u32,
    pub remained_penalty: u32,
    pub index_of_operation: u32,
    pub alternative_match_count: u32,
}

impl TraversedPosition{
    fn to_right_traversed_anchor(
        self,
        anchor_index: AnchorIndex,
        length_of_extension: u32,
        penalty_of_extension: u32,
    ) -> TraversedAnchorDep {
        TraversedAnchorDep {
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
        length_of_extension: u32,
        penalty_of_extension: u32,
    ) -> TraversedAnchorDep {
        TraversedAnchorDep {
            anchor_index,
            remained_length: length_of_extension - self.traversed_length_to_anchor_end,
            remained_penalty: penalty_of_extension - self.traversed_penalty_to_anchor_end,
            index_of_operation: self.index_of_operation,
            alternative_match_count: self.alternative_match_count,
        }
    }
}

impl PosTable {
    pub fn right_traversed_anchors(
        &self,
        traversed_positions: Vec<TraversedPosition>,
        anchor_pattern_index: u32,
        anchor_pattern_count: u32,
        target_start_index: u32,
        length_of_extension: u32,
        penalty_of_extension: u32,
        pattern_size: u32,
    ) -> Vec<TraversedAnchorDep> {
        traversed_positions.into_iter().map(|traversed_position| {
            let mut pattern_index = anchor_pattern_index + anchor_pattern_count + traversed_position.pattern_count_from_start_point;
            let mut target_position = target_start_index + traversed_position.traversed_record_length_to_anchor;

            let anchor_index_in_pattern = loop {
                let pattern_position = &self.0[pattern_index.as_usize()];
                let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, target_position);
                match anchor_index_in_pattern {
                    Ok(index) => {
                        break index as u32;
                    },
                    Err(_) => {
                        pattern_index -= 1;
                        target_position -= pattern_size;
                    },
                }
            };

            let traversed_anchor = traversed_position.to_right_traversed_anchor(
                (pattern_index, anchor_index_in_pattern),
                length_of_extension,
                penalty_of_extension,
            );
            traversed_anchor
        }).collect()
    }
    pub fn left_traversed_anchors(
        &self,
        traversed_positions: Vec<TraversedPosition>,
        anchor_pattern_index: u32,
        record_last_index: u32,
        length_of_extension: u32,
        penalty_of_extension: u32,
        pattern_size: u32,
    ) -> Vec<TraversedAnchorDep> {
        traversed_positions.into_iter().map(|traversed_position| {
            let mut pattern_index = anchor_pattern_index - traversed_position.pattern_count_from_start_point;
            let mut target_position = record_last_index - traversed_position.traversed_record_length_to_anchor;

            let anchor_index_in_pattern = loop {
                let pattern_position = &self.0[pattern_index.as_usize()];
                let anchor_index_in_pattern = AnchorPosition::binary_search_index(pattern_position, target_position);
                match anchor_index_in_pattern {
                    Ok(index) => {
                        break index as u32;
                    },
                    Err(_) => {
                        pattern_index += 1;
                        target_position += pattern_size;
                    },
                }
            };

            let traversed_anchor = traversed_position.to_left_traversed_anchor(
                (pattern_index, anchor_index_in_pattern),
                length_of_extension,
                penalty_of_extension,
            );
            traversed_anchor
        }).collect()
    }
}

impl AnchorPosition {
    #[inline]
    pub fn binary_search_index(pattern_position: &Vec<Self>, target_position: u32) -> Result<usize, usize> {
        pattern_position.binary_search_by_key(&target_position, |anchor_position| {
            anchor_position.target_position
        })
    }
}
