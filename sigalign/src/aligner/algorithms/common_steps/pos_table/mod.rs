use crate::core::{
    ReferenceInterface,
};
use ahash::AHashMap;

/**
Position Table: Sorted target positions by pattern
  - 1: Pattern index
  - 2: AnchorPosition sorted by position_in_target
If the locations of consecutive patterns are ungapped -> merge to one AnchorPosition
*/
// TODO: Reuse PosTable by caching it on the wave front cache
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PosTable(
    pub Vec<Vec<AnchorPosition>>
);
pub type AnchorIndex = (u32, u32);

/**
Position of anchor in target 
  - Restrict to u32
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorPosition {
    pub target_position: u32,
    pub pattern_count: u32,
}

impl PosTable {
    pub fn new_by_target_index<R: ReferenceInterface>(
        reference: &R,
        query: &[u8],
        pattern_size: u32,
    ) -> AHashMap<u32, Self> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size as usize;

        let mut pos_table_by_target_index: AHashMap<u32, Self> = AHashMap::new();

        (0..pattern_count).for_each(|pattern_index| {
            let qry_pos = pattern_index * pattern_size as usize;
            let pattern = &query[qry_pos..qry_pos+pattern_size as usize];
            
            let pattern_locations = reference.locate(pattern);

            pattern_locations.into_iter().for_each(|pattern_location| {
                match pos_table_by_target_index.get_mut(&pattern_location.target_index) {
                    Some(pos_table) => {
                        pos_table.add_new_positions(
                            pattern_index,
                            pattern_location.sorted_positions,
                        )
                    },
                    None => {
                        let mut new_pos_table = Self::new_empty(pattern_count);
                        new_pos_table.add_new_positions(
                            pattern_index,
                            pattern_location.sorted_positions,
                        );
                        pos_table_by_target_index.insert(pattern_location.target_index, new_pos_table);
                    }
                }
            });
        });

        pos_table_by_target_index.iter_mut().for_each(|(_, pos_table)| {
            pos_table.merge_ungapped_anchors(pattern_size);
        });

        pos_table_by_target_index
    }
    fn add_new_positions(
        &mut self,
        pattern_index: usize,
        sorted_target_positions: Vec<u32>,
    ) {
        self.0[pattern_index] = AnchorPosition::new_vec(sorted_target_positions);
    }
    fn new_empty(pattern_count: usize) -> Self {
        Self(vec![Vec::new(); pattern_count])
    }
    fn merge_ungapped_anchors(&mut self, pattern_size: u32) {
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
    fn new_vec(sorted_target_positions: Vec<u32>) -> Vec<Self> {
        sorted_target_positions.into_iter().map(|pos| {
            Self {
                target_position: pos,
                pattern_count: 1,
            }
        }).collect()
    }
    fn merge_right_to_left(left: &mut Vec<Self>, right: &mut Vec<Self>, pattern_size: u32) {
        let left_count = left.len();
        let mut right_count = right.len();

        if (left_count == 0) || (right_count == 0) {
            return
        }

        let mut left_index = 0;
        let mut right_index = 0;

        while (left_index < left_count) && (right_index < right_count) {
            let left_anchor_position = &mut left[left_index];
            let right_anchor_position = &right[right_index];
            let right_target_position = right_anchor_position.target_position;

            match (left_anchor_position.target_position + pattern_size).checked_sub(right_target_position) {
                Some(record_position_gap) => {
                    if record_position_gap == 0 {
                        let right_pattern_count = right_anchor_position.pattern_count;
                        left_anchor_position.pattern_count += right_pattern_count;

                        right.remove(right_index);

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
                        target_position: 20,
                        pattern_count: 2,
                    },
                    AnchorPosition {
                        target_position: 50,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        target_position: 80,
                        pattern_count: 3,
                    },
                ],
                vec![
                    AnchorPosition {
                        target_position: 10,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        target_position: 70,
                        pattern_count: 2,
                    },
                ],
                vec![
                    AnchorPosition {
                        target_position: 0,
                        pattern_count: 1,
                    },
                    AnchorPosition {
                        target_position: 150,
                        pattern_count: 1,
                    },
                ],
            ],
        );

        assert_eq!(pos_table, answer);
    }
}