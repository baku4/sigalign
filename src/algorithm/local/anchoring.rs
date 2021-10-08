use super::{Cutoff, MinPenaltyForPattern};
use super::{Reference, Sequence};
use super::{Anchors, Anchor, Estimation};

mod preset;

use std::collections::HashMap;

#[derive(Debug)]
pub struct AnchorsPreset {
    total_pattern_count: usize,
    matched_pattern_locations: Vec<PatternLocation>,
}

impl AnchorsPreset {
    pub fn new_by_record(
        reference: &dyn Reference,
        query: Sequence,
        pattern_size: usize,
    ) -> HashMap<usize, AnchorsPreset> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size;

        let mut anchors_preset_by_record: HashMap<usize, AnchorsPreset> = HashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * pattern_size;
            let pattern = &query[qry_pos..qry_pos+pattern_size];

            let reference_location = reference.locate(pattern, pattern_size);

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
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) { // -> Anchors { //TODO:
        let anchors_by_patterns = self.create_anchors_by_patterns(pattern_size, query_length, record_length, min_penalty_for_pattern);

        // Self::anchors_by_patterns_to_anchors(anchors_by_patterns) //TODO:
    }
    fn create_anchors_by_patterns(
        self,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) { // -> Vec<AnchorsByPattern> { //TODO:
        let matched_pattern_index_list = self.matched_pattern_index_list();
        let each_pattern_matches = EachPatternMatches::new(
            self.total_pattern_count,
            &matched_pattern_index_list
        );



    }
    fn matched_pattern_index_list(&self) -> Vec<usize> {
        self.matched_pattern_locations.iter().map(|pattern_location| {
            pattern_location.index
        }).collect()
    }
}

struct AnchorsByPattern {
    pattern_index: usize,
    anchors: Vec<Anchor>,
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
struct EstimationPerPattern {
    estimations: Vec<Estimation>,
}

impl EstimationPerPattern {
    fn new(
        total_pattern_count: usize,
        pattern_size: usize,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        matched_pattern_index_list: Vec<usize>,
    ) {
        let penalty_for_odd = min_penalty_for_pattern.odd;
        let penalty_for_even = min_penalty_for_pattern.even;

        let mut existence = vec![false; total_pattern_count];
        for &matched_pattern_index in &matched_pattern_index_list {
            existence[matched_pattern_index] = true;
        };

        let mut accumulated_penalty_from_right: Vec<usize> = vec![0; total_pattern_count];

        let mut start_index_to_fill = 0;
        let mut filled_pre_is_odd = false;

        for &matched_pattern_index in matched_pattern_index_list.iter().chain([total_pattern_count].iter()) {
            for i in (start_index_to_fill..matched_pattern_index).rev() {
                if filled_pre_is_odd {
                    accumulated_penalty_from_right[i] = penalty_for_even;
                    filled_pre_is_odd = false;
                } else {
                    accumulated_penalty_from_right[i] = penalty_for_odd;
                    filled_pre_is_odd = true;
                }
            }

            start_index_to_fill = matched_pattern_index + 1;
            filled_pre_is_odd = false;
        }

        #[cfg(test)]
        println!("{:?}", accumulated_penalty_from_right);

        Self::accumulate_reverse(&mut accumulated_penalty_from_right);

        #[cfg(test)]
        println!("{:?}", accumulated_penalty_from_right);

        let normalized_penalty_per_pattern = pattern_size as f32 * cutoff.penalty_per_length;

        let mut determinant_score_from_right :Vec<f32> = accumulated_penalty_from_right.iter().rev().enumerate().map(|(index, &penalty)| {
            index as f32 * normalized_penalty_per_pattern - penalty as f32
        }).collect(); // estimated length * penalty per length - estimated penalty
        determinant_score_from_right.reverse();

        #[cfg(test)]
        println!("{:?}", determinant_score_from_right);

        let mut last_max = f32::MIN;
        let mut index_of_last_max_from_right = 0;
        let start_index_of_pattern: Vec<usize> = determinant_score_from_right.into_iter().enumerate().map(|(index, score)| {
            if score >= last_max {
                last_max = score;
                index_of_last_max_from_right = index;
            }
            index_of_last_max_from_right
        }).collect();

        #[cfg(test)]
        println!("{:?}", start_index_of_pattern);

        let estimation_per_pattern: Vec<Estimation> = start_index_of_pattern.into_iter().enumerate().map(|(end_index, start_index)| {
            let pattern_between_start_end = end_index - start_index;
            let length = pattern_between_start_end * pattern_size + pattern_size - 1;
            let penalty = accumulated_penalty_from_right[end_index] - accumulated_penalty_from_right[start_index];
            Estimation {
                penalty,
                length,
            }
        }).collect();

        #[cfg(test)]
        println!("{:#?}", estimation_per_pattern);
    }
    fn accumulate_reverse(reverse: &mut Vec<usize>) {
        let mut accumulated_penalty = 0;
        reverse.iter_mut().rev().for_each(|value| {
            accumulated_penalty += *value;
            *value = accumulated_penalty;
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_estimation_penalty() {
        let total_pattern_count = 10;
        let min_penalty_for_pattern = MinPenaltyForPattern {
            odd: 4,
            even: 6,
        };
        let matched_pattern_index_list = vec![2, 3, 5, 6, 7, 8, 9];

        let estimation_per_pattern = EstimationPerPattern::new(
            total_pattern_count,
            5,
            &Cutoff { minimum_aligned_length: 100, penalty_per_length: 0.2 },
            &min_penalty_for_pattern,
            matched_pattern_index_list
        );
    }
}