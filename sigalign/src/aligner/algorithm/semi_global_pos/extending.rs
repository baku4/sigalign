use super::{PRECISION_SCALE, Cutoff, Penalties};
use super::{Sequence};
use super::{AlignmentOperation, AlignmentCase};

use super::{Extension, WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

use super::{AnchorTable, PatternAnchors, Anchor, AnchorState};

type AnchorIndex = (usize, usize); // pattern index, index in PatternAnchors

impl AnchorTable {
    pub fn extend(
        &mut self,
        record_seq: Sequence,
        query_seq: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
        scaled_margins_of_left: &Vec<i64>,
    ) {
        let mut anchor_indices_stack = self.anchor_indices_stack();

        loop {
            let last_anchor_index = anchor_indices_stack.last();

            match last_anchor_index {
                Some(anchor_index) => {
                    let extension_result = self.extend_and_evaluate_anchor(anchor_index);

                    match extension_result {
                        ExtensionResult::Done => {
                            anchor_indices_stack.pop();
                        },
                        ExtensionResult::ExtendNext(anchor_index) => {
                            anchor_indices_stack.push(anchor_index);
                        },
                    }
                },
                None => {
                    break;
                },
            }
        };
    }
    fn anchor_indices_stack(&self) -> Vec<AnchorIndex> {
        self.anchors_by_pattern.iter().enumerate().map(|(pattern_index, pattern_anchors)| {
            (0..pattern_anchors.sorted_anchors.len()).map(move |v| {
                (pattern_index, v)
            })
        }).flatten().rev().collect()
    }
    fn extend_and_evaluate_anchor(
        &mut self,
        anchor_index: &AnchorIndex,
        record_seq: Sequence,
        query_seq: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        wave_front: &mut WaveFront,
        scaled_margins_of_left: &Vec<i64>,
    ) -> ExtensionResult {
        let current_anchor = &mut self.anchors_by_pattern[anchor_index.0].sorted_anchors[anchor_index.1];

        match &current_anchor.state {
            // New
            AnchorState::New => {
                let record_position = current_anchor.record_position;
                let query_position = pattern_size * anchor_index.0;

                let record_slice = &record_seq[record_position + pattern_size..];
                let query_slice = &query_seq[query_position + pattern_size..];

                let delta = scaled_margins_of_left[anchor_index.0];
                let spare_penalty = calculate_spare_penalty_from_margin(
                    delta, pattern_size,
                    query_slice.len(), record_slice.len(),
                    penalties, cutoff,
                );

                wave_front.align_right_to_end_point(record_slice, query_slice, penalties, spare_penalty);

                if wave_front.is_success() {
                    
                } else {

                }
            },
            AnchorState::RightExtensionSuccess => {
                
            },
            AnchorState::RightExtensionFail => {
                
            },
            _ => {

            },
        }

        ExtensionResult::Done
    }
}

enum ExtensionResult {
    Done,
    ExtendNext(AnchorIndex),
}

impl WaveFront {
    fn is_success(&self) -> bool {
        match &self.end_point.k {
            Some(_) => true,
            None => false,
        }
    }
    fn backtrace_from_last_penalty(&self) {
        self.end_point.score
    }
}

fn calculate_spare_penalty_from_margin(
    scaled_margin_of_other_side: i64,
    anchor_size: usize,
    query_length_this_side: usize,
    record_length_this_side: usize,
    penalties: &Penalties,
    cutoff: &Cutoff,
) -> usize {
    i64::max(
        penalties.o as i64,
        (
            penalties.e as i64 * scaled_margin_of_other_side
            + cutoff.maximum_penalty_per_scale as i64 * (
                (
                    penalties.e * (
                        anchor_size + query_length_this_side.min(record_length_this_side)
                    )
                ) as i64 - penalties.o as i64
            )
        ) / (
            PRECISION_SCALE * penalties.e - cutoff.maximum_penalty_per_scale
        ) as i64
    ) as usize
}