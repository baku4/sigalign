use super::MinPenaltyForPattern;

pub struct AnchorsPreset {
    total_pattern_count: usize,
    matched_pattern_locations: Vec<PatternLocation>,
}

impl AnchorsPreset {
    pub fn new(pattern_count: usize) -> Self {
        Self {
            total_pattern_count: pattern_count,
            matched_pattern_locations: Vec::new(),
        }
    }
    pub fn add_new_position(&mut self, pattern_index: usize, record_positions: Vec<usize>) {
        let new_pattern_location = PatternLocation::new(pattern_index, record_positions);
        self.matched_pattern_locations.push(new_pattern_location);
    }
    pub fn to_anchors(self, record_length: usize, query_length:usize, pattern_size: usize, min_penalty_for_pattern: &MinPenaltyForPattern) {
        let mut anchors_from_query = AnchorsFromQuery::new(self.total_pattern_count);

        let matched_pattern_index_list: Vec<usize> = self.matched_pattern_locations.iter().map(|pattern_location| {
            pattern_location.index
        }).collect();
        let penalty_per_pattern = PenaltyPerPattern::new(
            self.total_pattern_count,
            min_penalty_for_pattern, 
            matched_pattern_index_list
        );

        for pattern_location in self.matched_pattern_locations {
            let pattern_index = pattern_location.index;
            let record_positions = pattern_location.record_positions;

            let new_anchors_from_pattern = AnchorsFromPattern::new_for_semi_global(
                pattern_index,
                pattern_size,
                query_length,
                record_length,
                record_positions,
                &penalty_per_pattern,
            );

            anchors_from_query.add_anchors_from_pattern(new_anchors_from_pattern);
        }
    }
}

#[derive(Debug)]
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

struct PatternLocation {
    index: usize,
    record_positions: Vec<usize>,
}

impl PatternLocation {
    fn new(pattern_index: usize, record_positions: Vec<usize>) -> Self {
        Self {
            index: pattern_index,
            record_positions,
        }
    }
}

struct AnchorsFromQuery {
    anchors_from_patterns: Vec<AnchorsFromPattern>,
}

impl AnchorsFromQuery {
    fn new(pattern_count: usize) -> Self {
        Self {
            anchors_from_patterns: Vec::with_capacity(pattern_count)
        }
    }
    fn add_anchors_from_pattern(&mut self, anchors_from_pattern: AnchorsFromPattern) {
        self.anchors_from_patterns.push(anchors_from_pattern)
    }
    fn generate_checkpoints_for_semi_global(&mut self) {
        let total_patterns_for_anchors = self.anchors_from_patterns.len();
        for i in (1..total_patterns_for_anchors).rev() {
            let (left, right) = self.anchors_from_patterns[i - 1..=i].split_at_mut(1);

            let left_anchors_from_pattern = &mut left[0];
            let right_anchors_from_pattern = &mut right[0];

            left_anchors_from_pattern.connect_to_right(right_anchors_from_pattern);
        }
    }
}

#[derive(Debug)]
struct AnchorsFromPattern {
    query_position: usize,
    anchors: Vec<Anchor>,
}

impl AnchorsFromPattern {
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

            let left_estimated_extension = ExtimatedExtension::new(left_min_penalty, left_min_length + left_unmatched_pattern_count);
            let right_estimated_extension = ExtimatedExtension::new(right_min_penalty, right_min_length + right_unmatched_pattern_count);

            Anchor {
                record_position,
                size: pattern_size,
                left_extension: Extension::Estimated(left_estimated_extension),
                right_extension: Extension::Estimated(right_estimated_extension),
                left_checkpoints: CheckPoints::empty(),
                right_checkpoints: CheckPoints::empty(),
                dropped: false,
            }
        }).collect();

        Self {
            query_position,
            anchors,
        }
    }
    fn connect_to_right(&mut self, right: &mut Self) {
        let left_pattern_position = self.query_position;
        let right_pattern_position = right.query_position;

        let mut left_anchor_index = 0;
        let mut right_anchor_index = 0;
        
        let left_anchors_count = self.anchors.len();
        let right_anchors_count = right.anchors.len();

        let mut to_remove_right_anchors_index: Vec<usize> = Vec::new();

        while (left_anchor_index < left_anchors_count) && (right_anchor_index < right_anchors_count) {
            let left_anchor = &mut self.anchors[left_anchor_index];
            let right_anchor = &mut right.anchors[right_anchor_index];
            
            let query_position_gap = left_pattern_position + left_anchor.size - right_pattern_position;
            
            if let Some(record_position_gap) = (left_anchor.record_position + left_anchor.size).checked_sub(right_anchor.record_position) {
                if (record_position_gap == 0) && (query_position_gap == 0) {
                    left_anchor.size += right_anchor.size;
                    left_anchor.right_checkpoints = right_anchor.right_checkpoints.clone();
                }
            }
        }
    }
}

#[derive(Debug)]
struct Anchor {
    record_position: usize,
    size: usize,
    left_extension: Extension,
    right_extension: Extension,
    left_checkpoints: CheckPoints,
    right_checkpoints: CheckPoints,
    dropped: bool,
}

impl Anchor {

}

#[derive(Debug)]
enum Extension {
    Estimated(ExtimatedExtension),
    Exact(ExactExtension),
}

#[derive(Debug)]
struct ExtimatedExtension {
    penalty: usize,
    length: usize,
}

impl ExtimatedExtension {
    fn new(penalty: usize, length: usize) -> Self {
        Self {
            penalty,
            length,
        }
    }
}

#[derive(Debug)]
struct ExactExtension {

}

#[derive(Debug, Clone)]
struct CheckPoints(Vec<usize>);

impl CheckPoints {
    fn empty() -> Self {
        Self(Vec::new())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_penalty_per_pattern() {
        let total_pattern_count = 10;
        let min_penalty_for_pattern = MinPenaltyForPattern {
            odd: 4,
            even: 6,
        };
        let matched_pattern_index_list = vec![2, 6, 9];

        let penalty_per_pattern = PenaltyPerPattern::new(total_pattern_count, &min_penalty_for_pattern, matched_pattern_index_list);
        
        let forward = penalty_per_pattern.forward;
        let reverse = penalty_per_pattern.reverse;

        assert_eq!(forward, vec![4, 6, 0, 4, 6, 4, 0, 6, 4, 0]);
        assert_eq!(reverse, vec![6, 4, 0, 4, 6, 4, 0, 4, 6, 0]);
    }

    #[test]
    fn print_new_anchors_from_pattern_for_semi_global() {
        let kmer = 15;
        let query_length = 200;

        let total_pattern_count = query_length / kmer; // 13

        let matched_pattern_index_list = vec![0, 3, 5, 6, 10]; 

        let pattern_index = matched_pattern_index_list[2]; // 5
        let pattern_size = kmer;

        let record_length = 1000;
        let record_positions = vec![10, 50, 200, 350];

        let min_penalty_for_pattern = MinPenaltyForPattern {
            odd: 4,
            even: 6,
        };
        
        let penalty_per_pattern = PenaltyPerPattern::new(
            total_pattern_count,
            &min_penalty_for_pattern,
            matched_pattern_index_list
        );

        println!("{:?}", penalty_per_pattern);

        let anchors_from_pattern = AnchorsFromPattern::new_for_semi_global(
            pattern_index,
            pattern_size,
            query_length,
            record_length,
            record_positions,
            &penalty_per_pattern,
        );
        
        print!("{:#?}", anchors_from_pattern);
    }
}