use super::{Cutoff, MinPenaltyForPattern};
use super::{ReferenceInterface, Sequence};
use super::{Anchors, Anchor};

use std::collections::HashMap;

#[derive(Debug)]
pub struct AnchorsPreset {
    total_pattern_count: usize,
    matched_pattern_locations: Vec<PatternLocation>,
}

impl AnchorsPreset {
    pub fn new_by_record(
        reference: &dyn ReferenceInterface,
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
        let matched_pattern_index_list = self.matched_pattern_index_list();
        let estimation_per_pattern = SparePenaltyPaddingPerPattern::new(self.total_pattern_count, pattern_size, cutoff, min_penalty_for_pattern, matched_pattern_index_list);

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
    fn matched_pattern_index_list(&self) -> Vec<usize> {
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
        estimation_per_pattern: &SparePenaltyPaddingPerPattern,
    ) -> Self {
        let query_position = pattern_index * pattern_size;

        let anchors: Vec<Anchor> = record_positions.into_iter().map(|record_position| {
            Anchor {
                query_position,
                record_position,
                size: pattern_size,
                spare_penalty_padding_of_left: estimation_per_pattern.spare_penalty_paddings[pattern_index].clone(),
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
                            left_anchor.spare_penalty_padding_of_left = right_anchor.spare_penalty_padding_of_left;
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

struct EachPatternMatches(Vec<bool>);

impl EachPatternMatches {
    fn new(
        total_pattern_count: usize,
        matched_pattern_index_list: &Vec<usize>,
    ) -> Self {
        let mut each_pattern_matches = vec![false; total_pattern_count];
        for &matched_pattern_index in matched_pattern_index_list {
            each_pattern_matches[matched_pattern_index] = true;
        };
        Self(each_pattern_matches)
    }
    fn count_unmatched_pattern(&self, start_index: usize, end_index: usize) -> usize {
        self.0[start_index..end_index].iter().filter(|&&v| !v).count()
    }
}

#[derive(Debug)]
struct SparePenaltyPaddingPerPattern {
    spare_penalty_paddings: Vec<f32>,
}

impl SparePenaltyPaddingPerPattern {
    fn new(
        total_pattern_count: usize,
        pattern_size: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        matched_pattern_index_list: Vec<usize>,
    ) -> Self {
        let penalty_for_odd = min_penalty_for_pattern.odd;
        let penalty_for_even = min_penalty_for_pattern.even;

        let normalized_penalty_cutoff_per_pattern = pattern_size as f32 * cutoff.penalty_per_length;
        let maximum_padding_to_next_pattern = (pattern_size-1) as f32 * cutoff.penalty_per_length;

        println!("maximum_padding_to_next_pattern: {:?}", maximum_padding_to_next_pattern);

        let mut existence = vec![false; total_pattern_count];
        for &matched_pattern_index in &matched_pattern_index_list {
            existence[matched_pattern_index] = true;
        };

        let mut accumulated_penalty_padding: Vec<usize> = vec![0; total_pattern_count];

        let mut start_index_to_fill = 0;
        let mut filled_pre_is_odd = false;

        for &matched_pattern_index in matched_pattern_index_list.iter().chain([total_pattern_count].iter()) {
            for i in (start_index_to_fill..matched_pattern_index).rev() {
                if filled_pre_is_odd {
                    accumulated_penalty_padding[i] = penalty_for_even;
                    filled_pre_is_odd = false;
                } else {
                    accumulated_penalty_padding[i] = penalty_for_odd;
                    filled_pre_is_odd = true;
                }
            }

            start_index_to_fill = matched_pattern_index + 1;
            filled_pre_is_odd = false;
        }

        #[cfg(test)]
        println!("{:?}", accumulated_penalty_padding);

        let mut accumulated_penalty_padding_per_pattern_from_right = Self::accumulate_with_normalized_padding(accumulated_penalty_padding, normalized_penalty_cutoff_per_pattern);
        accumulated_penalty_padding_per_pattern_from_right.reverse();

        // #[cfg(test)]
        // println!("{:?}", {
        //     let mut temp = accumulated_penalty_padding_per_pattern_from_right.clone();
        //     temp.reverse();
        //     temp
        // });
        println!("{:?}", accumulated_penalty_padding_per_pattern_from_right);

        let mut last_max = f32::MIN;
        let mut index_of_last_max = 0;
        let end_index_of_pattern: Vec<usize> = accumulated_penalty_padding_per_pattern_from_right.iter().enumerate().map(|(index, &score)| {
            if score >= last_max {
                last_max = score;
                index_of_last_max = index;
            }
            index_of_last_max
        }).collect();

        #[cfg(test)]
        println!("{:?}", &end_index_of_pattern);

        let mut spare_penalty_paddings: Vec<f32> = end_index_of_pattern.into_iter().enumerate().map(|(start_index, end_index)| {
            let mut spare_penalty_padding = &accumulated_penalty_padding_per_pattern_from_right[end_index] - &accumulated_penalty_padding_per_pattern_from_right[start_index];

            spare_penalty_padding += maximum_padding_to_next_pattern;

            spare_penalty_padding
        }).collect();

        spare_penalty_paddings[0] = 0.0; // First padding is always zero

        #[cfg(test)]
        println!("{:?}", spare_penalty_paddings);

        Self {
            spare_penalty_paddings: spare_penalty_paddings,
        }
    }
    fn accumulate_with_normalized_padding(
        penalty_per_pattern: Vec<usize>,
        normalized_penalty_cutoff_per_pattern: f32,
    ) -> Vec<f32> {
        let mut accumulated_penalty: f32 = 0.0;
        penalty_per_pattern.into_iter().rev().map(|value| {
            accumulated_penalty += normalized_penalty_cutoff_per_pattern - value as f32;
            accumulated_penalty
        }).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_spare_penalty_padding() {
        let total_pattern_count = 10;
        let min_penalty_for_pattern = MinPenaltyForPattern {
            odd: 4,
            even: 6,
        };
        let matched_pattern_index_list = vec![2, 3, 5, 9];

        let spare_penalty_padding = SparePenaltyPaddingPerPattern::new(
            total_pattern_count,
            5,
            &Cutoff { minimum_aligned_length: 100, penalty_per_length: 0.1 },
            &min_penalty_for_pattern,
            matched_pattern_index_list
        );
    }
}
