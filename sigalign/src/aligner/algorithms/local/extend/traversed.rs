use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
    }
};
use super::{AnchorTable, Anchor, AnchorIndex};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use super::Vpc;
use super::TraversedPosition;
use ahash::AHashSet;

#[derive(Debug, Clone)]
pub struct TraversedAnchor {
    pub anchor_index: AnchorIndex,
    pub scaled_penalty_delta_from_the_end: u32,
    pub partial_operation_index: u32,
    pub alternative_match_count: u32,
    pub to_skip: bool,
}
#[derive(Debug, Clone)]
pub struct PositionSymbol {
    
}

#[inline]
pub fn get_right_traversed_anchors(
    anchor_table: &AnchorTable,
    traversed_positions: &mut Vec<TraversedPosition>,
    right_spare_penalty_by_pattern_index: &Vec<u32>,
    base_pattern_index: u32,
    base_target_position: u32,
    pattern_size: u32,
) -> Vec<TraversedAnchor> {
    let base_spare_penalty = right_spare_penalty_by_pattern_index[base_pattern_index as usize];

    traversed_positions.iter_mut().map(|traversed_position| {
        let mut pattern_index = base_pattern_index + traversed_position.estimated_additive_pattern_index;
        let mut target_position = base_target_position + traversed_position.estimated_additive_target_position;
        let anchor_index_in_pattern = loop {
            let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
            match binary_search(
                anchors_by_pattern,
                target_position,
            ) {
                Ok(v) => {
                    break v as u32;
                },
                Err(_) => {
                    pattern_index -= 1;
                    target_position -= pattern_size;
                    panic!("")
                    // traversed_position 
                },
            }
        };
        
        let to_skip = right_spare_penalty_by_pattern_index[pattern_index as usize] <= base_spare_penalty - traversed_position.penalty_from_the_start;

        TraversedAnchor {
            anchor_index: (pattern_index, anchor_index_in_pattern),
            scaled_penalty_delta_from_the_end: traversed_position.scaled_penalty_delta_from_the_end,
            partial_operation_index: traversed_position.partial_operation_index,
            alternative_match_count: traversed_position.alternative_match_count,
            to_skip,
        }
    }).collect()
}

#[inline(always)]
fn binary_search(
    anchors_by_pattern: &Vec<Anchor>,
    target_position: u32,
) -> Result<usize, usize> {
    anchors_by_pattern.binary_search_by_key(&target_position, |anchor| {
        anchor.target_position
    })
}