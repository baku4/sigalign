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
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, SparePenaltyCalculator};
use super::Vpc;
use super::TraversedPosition;
use ahash::AHashSet;

use super::SideExtension;

#[derive(Debug, Clone)]
pub struct TraversedAnchor {
    pub anchor_index: AnchorIndex,
    pub partial_operation_index: u32,
    pub alternative_match_count: u32,
    pub scaled_penalty_delta_from_the_end: i64,
}
#[derive(Debug, Clone)]
pub struct PositionSymbol {
    
}

// FIXME:
// Check if `binary_search` can always find traversed anchor
// Unmut the traversed_positions
#[inline]
pub fn get_left_traversed_anchors_tagging_skip_info(
    anchor_table: &mut AnchorTable,
    traversed_positions: &mut Vec<TraversedPosition>,
    spare_penalty_calculator: &SparePenaltyCalculator,
    scaled_penalty_delta_assuming_leftmost_anchor: u32,
    base_pattern_index: u32,
    base_pattern_count: u32,
    base_target_position: u32,
    side_extension: &SideExtension,
) -> Vec<TraversedAnchor> {
    let base_spare_penalty = spare_penalty_calculator.get_left_spare_penalty(
        base_pattern_index,
        base_pattern_count,
    );

    traversed_positions.iter_mut().map(|traversed_position| {
        let pattern_index = base_pattern_index - traversed_position.estimated_additive_pattern_index;
        let target_position = base_target_position - traversed_position.estimated_additive_target_position;
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
                    panic!("")
                },
            }
        };

        // FIXME: check if the traversed_position is collect
        {
            let mut reversed_operation_of_traversed = {
                let vec = &side_extension.reversed_operations;
                vec[..=traversed_position.partial_operation_index as usize].to_vec()
            };
            let last = reversed_operation_of_traversed.last_mut().unwrap();
            last.count = traversed_position.alternative_match_count;
            if traversed_position.alternative_match_count == 0 {
                reversed_operation_of_traversed.pop(); // Can be
            }
            let (length, penalty, penalty_delta) = get_info_from_ops(&reversed_operation_of_traversed);
            if penalty_delta != traversed_position.scaled_penalty_delta_from_the_end {
                println!("base_pattern_index: {:?}", base_pattern_index);
                println!("anchor_table: {:#?}", anchor_table);
                println!("traversed_position: {:#?}", traversed_position);
                println!("side_extension: {:?}", side_extension);
                println!("reversed_operation_of_traversed: {:?}", reversed_operation_of_traversed);
                println!("penalty_delta: {}", penalty_delta);
                println!("traversed_position.scaled_penalty_delta_from_the_end: {}", traversed_position.scaled_penalty_delta_from_the_end);
                panic!("")
            }
            assert_eq!(side_extension.penalty - penalty, traversed_position.penalty_from_the_start);
        }
        
        // Check if extending can be skipped
        //   - Left extension assuming the rightmost anchor is already included in the extension of right anchor
        let can_skip_extending_to_the_left = {
            let spare_penalty_of_traversed = spare_penalty_calculator.get_left_spare_penalty(
                pattern_index,
                anchor_table.0[pattern_index as usize][anchor_index_in_pattern as usize].pattern_count,
            );
            spare_penalty_of_traversed <= base_spare_penalty - traversed_position.penalty_from_the_start
        };
        //   - Traversed anchor always has the connected anchor in left side
        let can_skip_extending_to_the_right = (scaled_penalty_delta_assuming_leftmost_anchor as i64) < traversed_position.scaled_penalty_delta_from_the_end;

        let traversed_anchor = &mut anchor_table.0[pattern_index as usize][anchor_index_in_pattern as usize];
        if can_skip_extending_to_the_left {
            traversed_anchor.skip_extending_to_the_left = true;
        };
        if can_skip_extending_to_the_right {

            traversed_anchor.skip_extending_to_the_right = true;
        };

        TraversedAnchor {
            anchor_index: (pattern_index, anchor_index_in_pattern),
            scaled_penalty_delta_from_the_end: traversed_position.scaled_penalty_delta_from_the_end,
            partial_operation_index: traversed_position.partial_operation_index,
            alternative_match_count: traversed_position.alternative_match_count,
        }
    }).collect()
}
#[inline]
pub fn get_right_traversed_anchors_tagging_skip_info(
    anchor_table: &mut AnchorTable,
    traversed_positions: &mut Vec<TraversedPosition>,
    spare_penalty_calculator: &SparePenaltyCalculator,
    base_pattern_index: u32,
    base_target_position: u32,
    side_extension: &SideExtension,
) -> Vec<TraversedAnchor> {
    let base_spare_penalty = spare_penalty_calculator.get_right_spare_penalty(base_pattern_index);

    traversed_positions.iter_mut().filter_map(|traversed_position| {
        let pattern_index = base_pattern_index + traversed_position.estimated_additive_pattern_index;
        let target_position = base_target_position + traversed_position.estimated_additive_target_position;

        let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
        match binary_search(
            anchors_by_pattern,
            target_position,
        ) {
            Ok(anchor_index_in_pattern) => {
                // Check if extending can be skipped
                //   - Right extension assuming the leftmost anchor is already included in the extension of left anchor
                let can_skip_extending_to_the_right = {
                    let spare_penalty_of_traversed = spare_penalty_calculator.get_right_spare_penalty(pattern_index);
                    spare_penalty_of_traversed + traversed_position.penalty_from_the_start <= base_spare_penalty
                };
                if can_skip_extending_to_the_right {
                    anchor_table.0[pattern_index as usize][anchor_index_in_pattern].skip_extending_to_the_right = true;
                }

                // FIXME: check if the traversed_position is collect
                {
                    let mut reversed_operation_of_traversed = {
                        let vec = &side_extension.reversed_operations;
                        vec[..=traversed_position.partial_operation_index as usize].to_vec()
                    };
                    let last = reversed_operation_of_traversed.last_mut().unwrap();
                    last.count = traversed_position.alternative_match_count;
                    if traversed_position.alternative_match_count == 0 {
                        reversed_operation_of_traversed.pop(); // Can be
                    }
                    let (length, penalty, penalty_delta) = get_info_from_ops(&reversed_operation_of_traversed);
                    if penalty_delta != traversed_position.scaled_penalty_delta_from_the_end {
                        println!("base_pattern_index: {:?}", base_pattern_index);
                        println!("anchor_table: {:#?}", anchor_table);
                        println!("traversed_position: {:#?}", traversed_position);
                        println!("side_extension: {:?}", side_extension);
                        println!("reversed_operation_of_traversed: {:?}", reversed_operation_of_traversed);
                        println!("penalty_delta: {}", penalty_delta);
                        println!("traversed_position.scaled_penalty_delta_from_the_end: {}", traversed_position.scaled_penalty_delta_from_the_end);
                        panic!("")
                    }
                    assert_eq!(side_extension.penalty - penalty, traversed_position.penalty_from_the_start);
                }

                Some(TraversedAnchor {
                    anchor_index: (pattern_index, anchor_index_in_pattern as u32),
                    scaled_penalty_delta_from_the_end: traversed_position.scaled_penalty_delta_from_the_end,
                    partial_operation_index: traversed_position.partial_operation_index,
                    alternative_match_count: traversed_position.alternative_match_count,
                })
            },
            Err(_) => {
                // Traversed anchor is merged into the right before anchor
                println!("Traversed anchor is merged into the right before anchor (right)");
                None
            },
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


fn get_info_from_ops(
    ops: &Vec<AlignmentOperations>,
) -> (u32, u32, i64) {
    let px = 2;
    let po = 3;
    let pe = 1;
    let maximum_scaled_penalty_per_length = ((PREC_SCALE as f32 * 0.1) / 2 as f32) as i64;

    let mut length = 0;
    let mut penalty = 0;
    ops.iter().for_each(|v| {
        match v.operation {
            AlignmentOperation::Match => {
                length += v.count;
            },
            AlignmentOperation::Subst => {
                length += v.count;
                penalty += v.count * px;
            },
            AlignmentOperation::Deletion => {
                length += v.count;
                penalty += v.count * pe + po;
            },
            AlignmentOperation::Insertion => {
                length += v.count;
                penalty += v.count * pe + po;
            },
        }
    });
    (length, penalty, length as i64 * maximum_scaled_penalty_per_length - (penalty * PREC_SCALE) as i64)
}