use ahash::AHashMap;

use crate::core::{BufferedPatternLocator, PatternLocation};
use super::Reference;
use super::pattern_index::PatternIndex;
use super::sequence_storage::SequenceStorage;

impl<I, S> BufferedPatternLocator for Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    type Buffer = S::Buffer;

    #[inline]
    fn locate(&self, pattern: &[u8], sorted_target_indices: &[u32]) -> Vec<PatternLocation> {
        let sorted_positions = self.pattern_index.get_sorted_positions(pattern);
        // TODO: Applying cap is valuable?
        let mut positions_by_target: AHashMap<u32, Vec<u32>> = AHashMap::new();

        let search_range_count = sorted_target_indices.len();

        let mut size;
        let mut left;
        let mut right;
        let mut mid = 0;
        let mut index;

        for position in sorted_positions {
            // reset
            right = search_range_count;
            left = mid;
            size = right - left;

            while left < right {
                mid = left + size / 2;
                index = sorted_target_indices[mid];
                
                let start = self.target_boundaries[index as usize];
                let end = self.target_boundaries[(index + 1) as usize];

                if position >= end {
                    left = mid + 1;
                } else if start > position {
                    right = mid;
                } else if position + pattern.len() as u32 <= end {
                    let ref_pos = position - start;
                    match positions_by_target.get_mut(&index) {
                        Some(v) => {
                            v.push(ref_pos);
                        },
                        None => {
                            positions_by_target.insert(index, vec![ref_pos]);
                        },
                    }
                    break;
                } else {
                    break;
                }
                
                size = right - left;
            }
        }

        positions_by_target.into_iter().map(|(target_index, positions)| {
            PatternLocation {
                target_index,
                sorted_positions: positions,
            }
        }).collect()
    }
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer) {
        self.sequence_storage.fill_buffer(target_index, buffer)
    }
}

impl<I, S> Reference<I, S> where
    I: PatternIndex,
    S: SequenceStorage,
{
    #[inline]
    pub fn locate_pattern(&self, pattern: &[u8], sorted_target_indices: &[u32]) -> Vec<PatternLocation> {
        self.locate(pattern, sorted_target_indices)
    }
    #[inline]
    pub fn get_sequence_buffer(&self) -> S::Buffer {
        self.sequence_storage.get_buffer()
    }
    #[inline]
    pub fn fill_sequence_buffer(&self, target_index: u32, buffer: &mut S::Buffer) {
        self.sequence_storage.fill_buffer(target_index, buffer)
    }
}

