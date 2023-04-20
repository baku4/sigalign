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
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker};
use super::Vpc;
use ahash::AHashSet;

#[derive(Debug, Clone)]
pub struct TraversedAnchorDep {
    pub anchor_index: AnchorIndex,
    pub partial_operation_index: u32,
    pub alternative_match_count: u32,
    pub scaled_penalty_delta_from_the_end: i64,
}

// FIXME: To del

/*

#[inline]
pub fn get_left_traversed_anchors_tagging_skip_info(
    anchor_table: &mut AnchorTable,
    traversed_positions: &Vec<TraversedPositionDep>,
    spare_penalty_calculator: &DepSparePenaltyCalculator,
    scaled_penalty_delta_assuming_leftmost_anchor: u32,
    base_pattern_index: u32,
    base_pattern_count: u32,
    base_target_position: u32,
    sequence_end_is_reached: bool,
) -> Vec<TraversedAnchorDep> {
    let base_spare_penalty = spare_penalty_calculator.get_left_spare_penalty(
        base_pattern_index,
        base_pattern_count,
    );

    traversed_positions.iter().map(|traversed_position| {
        let pattern_index = base_pattern_index - traversed_position.estimated_additive_pattern_index;
        let target_position = base_target_position - traversed_position.estimated_additive_target_position;
        let anchor_index_in_pattern = {
            let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
            match binary_search(
                anchors_by_pattern,
                target_position,
            ) {
                Ok(v) => {
                    v
                },
                Err(_) => {
                    panic!("")
                },
            }
        };

        // Check if extending can be skipped
        let traversed_anchor = &anchor_table.0[pattern_index as usize][anchor_index_in_pattern];
        // (1) Left
        if !traversed_anchor.skip_extending_to_the_left {
            let can_skip_extending_to_the_left = if sequence_end_is_reached {
                // Skip always
                true
            } else {
                // Left extension assuming the rightmost anchor is already included in the extension of right anchor
                let spare_penalty_of_traversed = spare_penalty_calculator.get_left_spare_penalty(
                    pattern_index,
                    anchor_table.0[pattern_index as usize][anchor_index_in_pattern as usize].pattern_count,
                );
                spare_penalty_of_traversed <= base_spare_penalty - traversed_position.penalty_from_the_start
            };
            if can_skip_extending_to_the_left {
                unsafe {
                    let mutable_ref = &mut *(traversed_anchor as *const Anchor as *mut Anchor);
                    mutable_ref.skip_extending_to_the_left = true;
                }
            };
        }
        // (2) Right
        if !traversed_anchor.skip_extending_to_the_right {
            if (scaled_penalty_delta_assuming_leftmost_anchor as i64) < traversed_position.scaled_penalty_delta_from_the_end {
                unsafe {
                    let mutable_ref = &mut *(traversed_anchor as *const Anchor as *mut Anchor);
                    mutable_ref.skip_extending_to_the_right = true;
                }
            }
        }

        TraversedAnchorDep {
            anchor_index: (pattern_index, anchor_index_in_pattern as u32),
            scaled_penalty_delta_from_the_end: traversed_position.scaled_penalty_delta_from_the_end,
            partial_operation_index: traversed_position.partial_operation_index,
            alternative_match_count: traversed_position.alternative_match_count,
        }
    }).collect()
}
#[inline]
pub fn get_right_traversed_anchors_tagging_skip_info(
    anchor_table: &mut AnchorTable,
    traversed_positions: &Vec<TraversedPositionDep>,
    spare_penalty_calculator: &DepSparePenaltyCalculator,
    base_pattern_index: u32,
    base_target_position: u32,
    sequence_end_is_reached: bool,
) -> Vec<TraversedAnchorDep> {
    let base_spare_penalty = spare_penalty_calculator.get_right_spare_penalty(base_pattern_index);

    traversed_positions.iter().filter_map(|traversed_position| {
        let pattern_index = base_pattern_index + traversed_position.estimated_additive_pattern_index;
        let target_position = base_target_position + traversed_position.estimated_additive_target_position;

        let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
        match binary_search(
            anchors_by_pattern,
            target_position,
        ) {
            Ok(anchor_index_in_pattern) => {
                let traversed_anchor = &anchor_table.0[pattern_index as usize][anchor_index_in_pattern];
                // Check if extending can be skipped
                // (1) Right
                if !traversed_anchor.skip_extending_to_the_right {
                    let can_skip_extending_to_the_right = if sequence_end_is_reached {
                        // Skip always
                        true
                    } else {
                        // Right extension assuming the leftmost anchor is already included in the extension of left anchor
                        let spare_penalty_of_traversed = spare_penalty_calculator.get_right_spare_penalty(pattern_index);
                        spare_penalty_of_traversed + traversed_position.penalty_from_the_start <= base_spare_penalty
                    };
                    if can_skip_extending_to_the_right {
                        unsafe {
                            let mutable_ref = &mut *(traversed_anchor as *const Anchor as *mut Anchor);
                            mutable_ref.skip_extending_to_the_right = true;
                        }
                    }
                }                

                Some(TraversedAnchorDep {
                    anchor_index: (pattern_index, anchor_index_in_pattern as u32),
                    scaled_penalty_delta_from_the_end: traversed_position.scaled_penalty_delta_from_the_end,
                    partial_operation_index: traversed_position.partial_operation_index,
                    alternative_match_count: traversed_position.alternative_match_count,
                })
            },
            Err(_) => {
                // Traversed anchor is merged into the right before anchor
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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_info_from_ops(
        ops: &Vec<AlignmentOperations>,
        penalties: &Penalty,
        maximum_scaled_penalty_per_length: u32,
    ) -> (u32, u32, i64) {
        let mut length = 0;
        let mut penalty = 0;
        ops.iter().for_each(|v| {
            match v.operation {
                AlignmentOperation::Match => {
                    length += v.count;
                },
                AlignmentOperation::Subst => {
                    length += v.count;
                    penalty += v.count * penalties.x;
                },
                AlignmentOperation::Deletion => {
                    length += v.count;
                    penalty += v.count * penalties.e + penalties.o;
                },
                AlignmentOperation::Insertion => {
                    length += v.count;
                    penalty += v.count * penalties.e + penalties.o;
                },
            }
        });
        (length, penalty, length as i64 * maximum_scaled_penalty_per_length as i64 - (penalty * PREC_SCALE) as i64)
    }
}


 */