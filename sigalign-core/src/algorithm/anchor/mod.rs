use crate::core::BufferedPatternLocator;
use ahash::AHashMap;

/**
Anchor Table: Sorted target positions by pattern
  - 1st Vec: Pattern index
  - 2nd Vec: Anchor sorted by target position
If the locations of consecutive patterns are ungapped -> merge to one Anchor

Anchor: position of alignment start point
  - target_position: leftmost position of the patterns
  - pattern_count: count of patterns
*/
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Anchor {
    pub target_position: u32,
    pub pattern_count: u32,
    pub extension_index: u32,
    pub to_skip: bool,
    pub used_to_results_as_leftmost_anchor: bool,
    pub used_to_results_as_rightmost_anchor: bool,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AnchorTable(
    pub Vec<Vec<Anchor>>
);
pub type AnchorIndex = (u32, u32);

impl AnchorTable {
    #[inline]
    pub fn new_by_target_index<L: BufferedPatternLocator>(
        pattern_locater: &L,
        query: &[u8],
        sorted_target_indices: &[u32],
        pattern_size: u32,
    ) -> AHashMap<u32, Self> {
        let qry_len = query.len();
        let pattern_count = qry_len / pattern_size as usize;

        let mut anchor_table_by_target_index: AHashMap<u32, Self> = AHashMap::new();

        (0..pattern_count).for_each(|pattern_index| {
            let qry_pos = pattern_index * pattern_size as usize;
            let pattern = &query[qry_pos..qry_pos+pattern_size as usize];
            
            let pattern_locations = pattern_locater.locate(pattern, sorted_target_indices);

            pattern_locations.into_iter().for_each(|pattern_location| {
                match anchor_table_by_target_index.get_mut(&pattern_location.target_index) {
                    Some(anchor_table) => {
                        anchor_table.add_new_positions(
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
                        anchor_table_by_target_index.insert(pattern_location.target_index, new_pos_table);
                    }
                }
            });
        });

        anchor_table_by_target_index.iter_mut().for_each(|(_, pos_table)| {
            pos_table.merge_ungapped_anchors(pattern_size);
        });

        anchor_table_by_target_index
    }
    fn add_new_positions(
        &mut self,
        pattern_index: usize,
        sorted_target_positions: Vec<u32>,
    ) {
        self.0[pattern_index] = Anchor::new_vec(sorted_target_positions);
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

            Anchor::merge_right_to_left(left, right, pattern_size);
        }
    }
}

impl Anchor {
    fn new_vec(sorted_target_positions: Vec<u32>) -> Vec<Self> {
        sorted_target_positions.into_iter().map(|pos| {
            Self {
                target_position: pos,
                pattern_count: 1,
                extension_index: 0,
                to_skip: false,
                used_to_results_as_leftmost_anchor: false,
                used_to_results_as_rightmost_anchor: false,
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
                Some(target_position_gap) => {
                    if target_position_gap == 0 {
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
