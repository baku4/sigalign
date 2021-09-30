use super::{Cutoff, Penalties, MinPenaltyForPattern};

mod preset;

pub use preset::AnchorsPreset;

const PATTERN_INDEX_GAP_FOR_CHECK_POINTS: usize = 3;

pub struct Anchors {
    anchors: Vec<Anchor>,
}

impl Anchors {
    fn create_checkpoints_between_anchors(
        &mut self,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        let allowed_gap_between_query_position: usize = pattern_size * PATTERN_INDEX_GAP_FOR_CHECK_POINTS;
        for right_first_anchor_index in 1..self.anchors.len() {
            let (left_anchors, right_anchors) = self.anchors.split_at_mut(right_first_anchor_index);

            let left_anchor = left_anchors.last_mut().unwrap();

            left_anchor.create_checkpoint_to_rights(right_anchors, right_first_anchor_index, allowed_gap_between_query_position, penalties, cutoff);
        }
    }
}

struct Anchor {
    query_position: usize,
    record_position: usize,
    size: usize,
    left_estimation: Estimation,
    right_estimation: Estimation,
    left_check_points: CheckPoints,
    right_check_points: CheckPoints,
    left_extension: Option<Extension>,
    right_extension: Option<Extension>,
    dropped: bool,
}

impl Anchor {
    fn create_checkpoint_to_rights(
        &mut self,
        right_anchors: &mut [Self],
        right_first_anchor_index: usize,
        allowed_gap_between_query_position: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
    ) {
        let left_anchor_index = right_first_anchor_index - 1;
        let mut right_anchor_index = left_anchor_index;

        for right_anchor in right_anchors {
            right_anchor_index += 1;

            let query_optional_gap = right_anchor.query_position.checked_sub(self.query_position + self.size);
            let can_be_connected = match query_optional_gap {
                None => {
                    continue
                },
                Some(query_gap) => {
                    if query_gap > allowed_gap_between_query_position {
                        break;
                    }

                    let record_optional_gap = right_anchor.record_position.checked_sub(self.record_position + self.size);
                    match record_optional_gap {
                        None => {
                            continue;
                        },
                        Some(record_gap) => {
                            let max_gap = record_gap.max(query_gap);
                            let min_gap = record_gap.min(query_gap);

                            let gap_count = max_gap - min_gap;

                            let min_penalty = if gap_count == 0 {
                                0
                            } else {
                                penalties.o + gap_count * penalties.e
                            };

                            let penalty = self.left_estimation.penalty + right_anchor.right_estimation.penalty + min_penalty;
                            let length = self.left_estimation.length + self.size + right_anchor.right_estimation.length + max_gap;
                            
                            let penalty_per_length = penalty as f32 / length as f32;

                            (length >= cutoff.minimum_aligned_length) && (penalty_per_length >= cutoff.penalty_per_length)
                        },
                    }
                },
            };

            if can_be_connected {
                self.right_check_points.add_new_checkpoint(right_anchor_index);
                right_anchor.left_check_points.add_new_checkpoint(left_anchor_index);
            }
        }
    }
}

struct Estimation {
    penalty: usize,
    length: usize,
}

impl Estimation {
    fn new(penalty: usize, length: usize) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

enum Extension {
    Own,
    Ref,
}

struct CheckPoints(Vec<usize>);

impl CheckPoints {
    fn empty() -> Self {
        Self(Vec::new())
    }
    fn add_new_checkpoint(&mut self, anchor_index: usize) {
        self.0.push(anchor_index);
    }
}