use super::{PRECISION_SCALE, Cutoff, MinPenaltyForPattern};
use super::{ReferenceInterface, Sequence, Reference, SequenceProvider};
use super::{Anchors, Anchor};

use std::collections::HashMap;

#[derive(Debug)]
pub struct AnchorsPreset {
    total_pattern_count: usize,
    matched_pattern_locations: Vec<PatternLocation>,
}

impl AnchorsPreset {
    pub fn new_by_record<'a, S: SequenceProvider<'a>>(
        reference: &Reference<'a, S>,
        query: Sequence,
        pattern_size: usize,
    ) -> HashMap<usize, AnchorsPreset> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size;

        let mut anchors_preset_by_record: HashMap<usize, AnchorsPreset> = HashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * pattern_size;
            let pattern = &query[qry_pos..qry_pos+pattern_size];

            let reference_location = reference.locate(pattern);

            for record_location in reference_location {
                match anchors_preset_by_record.get_mut(&record_location.record_index) {
                    Some(anchors_preset) => {
                        anchors_preset.add_new_position(pattern_index, record_location.positions)
                    },
                    None => {
                        let mut new_anchors_preset = Self::new(pattern_count);
                        new_anchors_preset.add_new_position(pattern_index, record_location.positions);
                        anchors_preset_by_record.insert(record_location.record_index, new_anchors_preset);
                    }
                }
            }
        }

        anchors_preset_by_record
    }
    fn new(total_pattern_count: usize) -> Self {
        Self {
            total_pattern_count,
            matched_pattern_locations: Vec::new(),
        }
    }
    fn add_new_position(&mut self, pattern_index: usize, record_positions: Vec<usize>) {
        let new_pattern_location = PatternLocation {
            index: pattern_index,
            record_positions,
        };
        self.matched_pattern_locations.push(new_pattern_location);
    }
    pub fn to_anchors(
        self,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> Anchors {
        let anchors_by_patterns = self.create_anchors_by_patterns(pattern_size, query_length, record_length, cutoff, min_penalty_for_pattern);

        Self::anchors_by_patterns_to_anchors(anchors_by_patterns)
    }
    fn create_anchors_by_patterns(
        self,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> Vec<AnchorsByPattern> {
        let sorted_matched_pattern_index_list = self.sorted_matched_pattern_index_list();
        let estimation_per_pattern = SparePenaltyDeterminantPerPattern::new(
            self.total_pattern_count,
            pattern_size,
            cutoff,
            min_penalty_for_pattern,
            sorted_matched_pattern_index_list,
        );

        let mut anchors_by_patterns: Vec<AnchorsByPattern> = self.matched_pattern_locations.into_iter().map(|pattern_location| {
            AnchorsByPattern::new(
                pattern_location.index,
                pattern_size,
                query_length,
                record_length,
                pattern_location.record_positions,
                &estimation_per_pattern,
            )
        }).collect();

        Self::concatenate_ungapped_anchors_by_patterns(&mut anchors_by_patterns);

        anchors_by_patterns
    }
    fn sorted_matched_pattern_index_list(&self) -> Vec<usize> {
        // Matched patterns are already sorted
        self.matched_pattern_locations.iter().map(|pattern_location| {
            pattern_location.index
        }).collect()
    }
    fn concatenate_ungapped_anchors_by_patterns(anchors_by_patterns: &mut Vec<AnchorsByPattern>) {
        for i in (1..anchors_by_patterns.len()).rev() {
            let (left, right) = anchors_by_patterns[i-1..=i].split_at_mut(1);

            let left_anchors_by_pattern = &mut left[0];
            let right_anchors_by_pattern = &mut right[0];

            right_anchors_by_pattern.consume_if_ungapped_to_left(left_anchors_by_pattern);
        }
    }
    fn anchors_by_patterns_to_anchors(anchors_by_patterns: Vec<AnchorsByPattern>) -> Anchors {
        let total_anchors_count: usize = anchors_by_patterns.iter().map(|anchors_by_pattern| {
            anchors_by_pattern.anchors.len()
        }).sum();

        let mut anchors: Vec<Anchor> = Vec::with_capacity(total_anchors_count);

        anchors_by_patterns.into_iter().for_each(|mut anchors_by_pattern| {
            anchors.append(&mut anchors_by_pattern.anchors)
        });

        Anchors {
            anchors
        }
    }
}

struct AnchorsByPattern {
    pattern_index: usize,
    anchors: Vec<Anchor>,
}
impl AnchorsByPattern {
    fn new(
        pattern_index: usize,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        record_positions: Vec<usize>,
        spare_penalty_determinant_per_pattern: &SparePenaltyDeterminantPerPattern,
    ) -> Self {
        let query_position = pattern_index * pattern_size;

        let anchors: Vec<Anchor> = record_positions.into_iter().map(|record_position| {
            Anchor {
                query_position,
                record_position,
                size: pattern_size,
                spare_penalty_determinant_of_left: spare_penalty_determinant_per_pattern.0[pattern_index],
                left_extension: None,
                right_extension: None,
                dropped: false,
            }
        }).collect();

        Self {
            pattern_index,
            anchors,
        }
    }
    fn consume_if_ungapped_to_left(&mut self, left: &mut Self) {
        let left_pattern_index = left.pattern_index;
        let right_pattern_index = self.pattern_index;

        if left_pattern_index + 1 == right_pattern_index {
            let mut left_anchor_index = 0;
            let mut right_anchor_index = 0;

            let left_anchors_count = left.anchors.len();
            let right_anchors_count = self.anchors.len();

            let mut to_remove_right_anchors_index: Vec<usize> = Vec::new();

            while (left_anchor_index < left_anchors_count) && (right_anchor_index < right_anchors_count) {
                let left_anchor = &mut left.anchors[left_anchor_index];
                let right_anchor = &mut self.anchors[right_anchor_index];

                match (left_anchor.record_position + left_anchor.size).checked_sub(right_anchor.record_position) {
                    Some(record_position_gap) => {
                        if record_position_gap == 0 { // Right record position == Left record position
                            to_remove_right_anchors_index.push(right_anchor_index);
                            left_anchor.size += right_anchor.size;
                        } else { // Right record position > Left record position
                            left_anchor_index += 1;
                        }
                    },
                    None => { // Right record position < Left record position
                        right_anchor_index += 1;
                    },
                }
            }

            to_remove_right_anchors_index.iter().rev().for_each(|&to_remove_index| {
                self.anchors.remove(to_remove_index);
            })
        }
    }
}

#[derive(Debug)]
struct PatternLocation {
    index: usize,
    record_positions: Vec<usize>,
}

// Spare penalty determinant:
// penalty per scale * length - PRECISION_SCALE * penalty
#[derive(Debug)]
struct SparePenaltyDeterminantPerPattern(Vec<i64>);
// TODO: Unsigned integer can be used?
// TODO: Precalculate duplicated parameters within Aligner?

impl SparePenaltyDeterminantPerPattern {
    fn new(
        total_pattern_count: usize,
        pattern_size: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        sorted_matched_pattern_index_list: Vec<usize>,
    ) -> Self {
        let scaled_penalty_for_odd = (min_penalty_for_pattern.odd * PRECISION_SCALE) as i64;
        let scaled_penalty_for_even = (min_penalty_for_pattern.even * PRECISION_SCALE) as i64;

        let mut pattern_existence = vec![false; total_pattern_count];
        for &matched_pattern_index in &sorted_matched_pattern_index_list {
            pattern_existence[matched_pattern_index] = true;
        };
        pattern_existence.pop(); // A last element is unnecessary.

        // (spare penalty determinant, if use even number of consecutive penalties)
        let mut previous_cell = (0, true);

        let penalty_per_scale_for_one_length = cutoff.maximum_penalty_per_scale as i64;

        // pattern size * penalty per length
        let spare_penalty_determinant_for_matched_pattern = pattern_size as i64 * penalty_per_scale_for_one_length;
        let spare_penalty_determinant_to_right_before_previous_pattern = spare_penalty_determinant_for_matched_pattern - penalty_per_scale_for_one_length;
        let spare_penalty_determinant_continued_to_previous_pattern = spare_penalty_determinant_for_matched_pattern + penalty_per_scale_for_one_length;

        let mut spare_penalty_determinant_per_pattern: Vec<i64> = pattern_existence.into_iter().map(|exist| {
            if exist {
                let new_spare_penalty_determinant = previous_cell.0 + spare_penalty_determinant_for_matched_pattern;
                previous_cell = (new_spare_penalty_determinant, true);
                new_spare_penalty_determinant
            } else {
                if previous_cell.1 { // Previous pattern use EVEN number of consecutive penalties
                    let continued_spare_penalty_determinant = previous_cell.0 + spare_penalty_determinant_continued_to_previous_pattern - scaled_penalty_for_odd;
                    if continued_spare_penalty_determinant < spare_penalty_determinant_to_right_before_previous_pattern {
                        // Equal sign is not used because it gives more opportunity to the next pattern of using even penalty
                        previous_cell = (spare_penalty_determinant_to_right_before_previous_pattern, true);
                        spare_penalty_determinant_to_right_before_previous_pattern
                    } else {
                        previous_cell = (continued_spare_penalty_determinant, false);
                        continued_spare_penalty_determinant
                    }
                } else { // Previous pattern use ODD number of consecutive penalties
                    let continued_spare_penalty_determinant = previous_cell.0 + spare_penalty_determinant_continued_to_previous_pattern - scaled_penalty_for_even;
                    if continued_spare_penalty_determinant < spare_penalty_determinant_to_right_before_previous_pattern {
                        previous_cell = (spare_penalty_determinant_to_right_before_previous_pattern, true);
                        spare_penalty_determinant_to_right_before_previous_pattern
                    } else {
                        previous_cell = (continued_spare_penalty_determinant, true);
                        continued_spare_penalty_determinant
                    }
                }
            }
        }).collect();

        spare_penalty_determinant_per_pattern.insert(0, 0);

        Self(spare_penalty_determinant_per_pattern)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spare_penalty_determinant() {
        let total_pattern_count = 10;
        let min_penalty_for_pattern = MinPenaltyForPattern {
            odd: 6,
            even: 4,
        };
        let matched_pattern_index_list = vec![2, 3, 5, 9];

        let spare_penalty_determinant_per_pattern = SparePenaltyDeterminantPerPattern::new(
            total_pattern_count,
            25,
            &Cutoff {
                minimum_aligned_length: 100,
                maximum_penalty_per_scale: 1_000,
            },
            &min_penalty_for_pattern,
            matched_pattern_index_list
        );

        assert_eq!(vec![0, 24000, 24000, 49000, 74000, 40000, 65000, 31000, 24000, 24000], spare_penalty_determinant_per_pattern.0);
    }
}
