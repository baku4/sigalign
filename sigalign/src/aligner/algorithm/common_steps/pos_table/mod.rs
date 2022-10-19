use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceStorage,
};

use std::collections::HashMap;

pub type AnchorIndex = (usize, usize);

// Sorted record positions by pattern
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosTable(pub Vec<Vec<AnchorPosition>>);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorPosition {
    pub record_position: usize,
    pub pattern_count: usize,
}

impl PosTable {
    pub fn new_by_record<S: SequenceStorage>(
        reference: &Reference<S>,
        query: Sequence,
        pattern_size: usize,
    ) -> HashMap<usize, Self> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size;

        let mut pos_table_by_record: HashMap<usize, Self> = HashMap::new();

        for pattern_index in 0..pattern_count {
            let qry_pos = pattern_index * pattern_size;
            let pattern = &query[qry_pos..qry_pos+pattern_size];
            
            let reference_location = reference.locate(pattern);

            for record_location in reference_location {
                match pos_table_by_record.get_mut(&record_location.record_index) {
                    Some(pos_table) => {
                        pos_table.add_new_positions(pattern_index, record_location.positions)
                    },
                    None => {
                        let mut new_pos_table = Self::new_empty(pattern_count);
                        new_pos_table.add_new_positions(pattern_index, record_location.positions);
                        pos_table_by_record.insert(record_location.record_index, new_pos_table);
                    }
                }
            }
        }

        pos_table_by_record.iter_mut().for_each(|(_, pos_table)| {
            pos_table.merge_ungapped_anchors(pattern_size);
        });

        pos_table_by_record
    }

    // For New
    fn new_empty(pattern_count: usize) -> Self {
        Self(vec![Vec::new(); pattern_count])
    }
    fn add_new_positions(
        &mut self,
        pattern_index: usize,
        sorted_record_positions: Vec<usize>,
    ) {
        self.0[pattern_index] = AnchorPosition::new_of_vector(sorted_record_positions);
    }
    // For Merge
    fn merge_ungapped_anchors(&mut self, pattern_size: usize) {
        let pattern_count = self.0.len();

        for right_index in (1..pattern_count).rev() {
            let (splitted_left, splitted_right) = self.0.split_at_mut(right_index);

            let left = &mut splitted_left[right_index-1];
            let right = &mut splitted_right[0];

            AnchorPosition::merge_right_to_left(left, right, pattern_size);
        }
    }
}

impl AnchorPosition {
    fn new_of_vector(sorted_record_positions: Vec<usize>) -> Vec<Self> {
        sorted_record_positions.into_iter().map(|pos| {
            Self {
                record_position: pos,
                pattern_count: 1,
            }
        }).collect()
    }
    fn merge_right_to_left(left_anchor_positions: &mut Vec<Self>, right_anchor_positions: &mut Vec<Self>, pattern_size: usize) {
        let left_count = left_anchor_positions.len();
        let mut right_count = right_anchor_positions.len();

        if (left_count == 0) || (right_count == 0) {
            return
        }

        let mut left_index = 0;
        let mut right_index = 0;

        while (left_index < left_count) && (right_index < right_count) {
            let left_anchor_position = &mut left_anchor_positions[left_index];
            let right_anchor_position = &right_anchor_positions[right_index];
            let right_record_position = right_anchor_position.record_position;

            match (left_anchor_position.record_position + pattern_size).checked_sub(right_record_position) {
                Some(record_position_gap) => {
                    if record_position_gap == 0 {
                        let right_pattern_count = right_anchor_position.pattern_count;
                        left_anchor_position.pattern_count += right_pattern_count;

                        right_anchor_positions.remove(right_index);

                        left_index += 1;
                        right_count -= 1;
                    } else {
                        right_index += 1;
                    }
                },
                None => {
                    left_index += 1;
                },
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_ungapped_anchors_for_pos_table() {
        let mut pos_table = PosTable::new_empty(3);

        let pattern_size = 10;

        pos_table.add_new_positions(0, vec![20, 50, 80]);
        pos_table.add_new_positions(1, vec![10, 30, 70, 90]);
        pos_table.add_new_positions(2, vec![0, 80, 100, 150]);

        pos_table.merge_ungapped_anchors(pattern_size);

        let answer = PosTable(
            vec![
                vec![
                    AnchorPosition {
                        record_position: 20,
                        pattern_count: 2,
                    },
                    AnchorPosition {
                        record_position: 50,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        record_position: 80,
                        pattern_count: 3,
                    },
                ],
                vec![
                    AnchorPosition {
                        record_position: 10,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        record_position: 70,
                        pattern_count: 2,
                    },
                ],
                vec![
                    AnchorPosition {
                        record_position: 0,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        record_position: 150,
                        pattern_count: 1,
                    },
                ],
            ],
        );

        assert_eq!(pos_table, answer);
    }
}