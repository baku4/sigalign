use super::{Anchor, Alignment, Estimation, CheckPoints, MinPenaltyForPattern};

struct AnchorsPreset {
    total_pattern_count: usize,
    matched_pattern_locations: Vec<PatternLocation>,
}

impl AnchorsPreset {
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
    fn to_anchors(
        self,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) {
        let mut anchors_by_patterns = self.create_anchors_by_patterns(pattern_size, query_length, record_length, min_penalty_for_pattern);

        for i in (1..anchors_by_patterns.len()).rev() {
            let (left, right) = anchors_by_patterns[i-1..=i].split_at_mut(1);

            let left_anchors_by_pattern = &mut left[0];
            let right_anchors_by_pattern = &mut right[0];

            right_anchors_by_pattern.consume_if_ungapped_to_left(left_anchors_by_pattern);
        }

    }
    fn create_anchors_by_patterns(
        self,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> Vec<AnchorsByPattern> {
        let matched_pattern_index_list = self.matched_pattern_index_list();
        let penalty_per_pattern = PenaltyPerPattern::new(
            self.total_pattern_count,
            min_penalty_for_pattern,
            matched_pattern_index_list
        );

        self.matched_pattern_locations.into_iter().map(|pattern_location| {
            AnchorsByPattern::new_for_semi_global(
                pattern_location.index,
                pattern_size,
                query_length,
                record_length,
                pattern_location.record_positions,
                &penalty_per_pattern
            )
        }).collect()
    }
    fn matched_pattern_index_list(&self) -> Vec<usize> {
        self.matched_pattern_locations.iter().map(|pattern_location| {
            pattern_location.index
        }).collect()
    }
}

struct AnchorsByPattern{
    pattern_index: usize,
    anchors: Vec<Anchor>,
}

impl AnchorsByPattern {
    fn new_for_semi_global(
        pattern_index: usize,
        pattern_size: usize,
        query_length: usize,
        record_length: usize,
        record_positions: Vec<usize>,
        penalty_per_pattern: &PenaltyPerPattern,
    ) -> Self {
        let query_position = pattern_index * pattern_size;
        let left_query_length = query_position;
        let right_query_length = query_length - query_position - pattern_size;

        let anchors: Vec<Anchor> = record_positions.into_iter().map(|record_position| {
            let left_record_length = record_position;
            let right_record_length = record_length - record_position - pattern_size;

            let left_min_length = left_query_length.min(left_record_length);
            let left_pattern_count = left_min_length / pattern_size;
            let right_min_length = right_query_length.min(right_record_length);
            let right_pattern_count = right_min_length / pattern_size;

            let left_pattern_start_index = pattern_index - left_pattern_count;
            let left_pattern_end_index = pattern_index;
            let right_pattern_start_index = pattern_index + 1;
            let right_pattern_end_index = pattern_index + right_pattern_count + 1;

            let left_unmatched_pattern_count = penalty_per_pattern.count_unmatched_pattern(left_pattern_start_index, left_pattern_end_index);
            let right_unmatched_pattern_count = penalty_per_pattern.count_unmatched_pattern(right_pattern_start_index, right_pattern_end_index);

            let left_min_penalty = penalty_per_pattern.minimum_penalty_of_left(left_pattern_start_index, left_pattern_end_index);
            let right_min_penalty = penalty_per_pattern.minimum_penalty_of_right(right_pattern_start_index, right_pattern_end_index);

            let left_estimation = Estimation::new(left_min_penalty, left_min_length + left_unmatched_pattern_count);
            let right_estimation = Estimation::new(right_min_penalty, right_min_length + right_unmatched_pattern_count);

            Anchor {
                query_position,
                record_position,
                size: pattern_size,
                left_alignment: Alignment::Estimated(left_estimation),
                left_check_points: CheckPoints::empty(),
                right_alignment: Alignment::Estimated(right_estimation),
                right_check_points: CheckPoints::empty(),
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

struct PatternLocation {
    index: usize,
    record_positions: Vec<usize>,
}

struct PenaltyPerPattern {
    existence: Vec<bool>,
    forward: Vec<usize>,
    reverse: Vec<usize>,
}

impl PenaltyPerPattern {
    fn new(
        total_pattern_count: usize,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        matched_pattern_index_list: Vec<usize>,
    ) -> Self {
        let penalty_for_odd = min_penalty_for_pattern.odd;
        let penalty_for_even = min_penalty_for_pattern.even;

        let mut existence = vec![false; total_pattern_count];
        for &matched_pattern_index in &matched_pattern_index_list {
            existence[matched_pattern_index] = true;
        };

        let mut forward = vec![0; total_pattern_count];
        let mut reverse = vec![0; total_pattern_count];
        
        let mut next_index_to_fill = 0;
        let mut filled_pre_is_odd = false;

        for &matched_pattern_index in matched_pattern_index_list.iter().chain([total_pattern_count].iter()) {
            let num_of_patterns_to_fill_is_even = (matched_pattern_index - next_index_to_fill) % 2 == 0;
            
            if num_of_patterns_to_fill_is_even {
                for i in next_index_to_fill..matched_pattern_index {
                    if filled_pre_is_odd {
                        forward[i] = penalty_for_even;
                        reverse[i] = penalty_for_odd;
                        filled_pre_is_odd = false;
                    } else {
                        forward[i] = penalty_for_odd;
                        reverse[i] = penalty_for_even;
                        filled_pre_is_odd = true;
                    }
                }
            } else {
                for i in next_index_to_fill..matched_pattern_index {
                    if filled_pre_is_odd {
                        forward[i] = penalty_for_even;
                        reverse[i] = penalty_for_even;
                        filled_pre_is_odd = false;
                    } else {
                        forward[i] = penalty_for_odd;
                        reverse[i] = penalty_for_odd;
                        filled_pre_is_odd = true;
                    }
                }
            }

            next_index_to_fill = matched_pattern_index + 1;
        }

        Self {
            existence,
            forward,
            reverse,
        }
    }
    fn count_unmatched_pattern(&self, start_index: usize, end_index: usize) -> usize {
        self.existence[start_index..end_index].iter().filter(|&&v| !v).count()
    }
    fn minimum_penalty_of_left(&self, start_index: usize, end_index: usize) -> usize {
        self.reverse[start_index..end_index].iter().sum()
    }
    fn minimum_penalty_of_right(&self, start_index: usize, end_index: usize) -> usize {
        self.forward[start_index..end_index].iter().sum()
    }
}
