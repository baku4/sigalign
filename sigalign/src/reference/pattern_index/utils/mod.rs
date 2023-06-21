use crate::core::SeqLen;
use super::PatternLocation;
use ahash::AHashMap;

#[inline(always)]
pub fn sorted_positions_to_pattern_location<L: SeqLen>(
    sorted_positions: &Vec<L>,
    boundaries: &Vec<L>,
    search_range: &Vec<u32>,
    pattern_size: u32,
) -> Vec<PatternLocation> {
    // TODO: Applying cap is valuable?
    let mut positions_by_target: AHashMap<u32, Vec<u32>> = AHashMap::new();

    let search_range_count = search_range.len();

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
            index = search_range[mid];
            
            let start = boundaries[index as usize];
            let end = boundaries[(index + 1) as usize];

            if *position >= end {
                left = mid + 1;
            } else if start > *position {
                right = mid;
            } else {
                if (*position + L::from_u32(pattern_size)) <= end {
                    let ref_pos = *position - start;
                    match positions_by_target.get_mut(&index) {
                        Some(v) => {
                            v.push(ref_pos.as_u32());
                        },
                        None => {
                            positions_by_target.insert(index, vec![ref_pos.as_u32()]);
                        },
                    }
                    break;
                } else {
                    break;
                }
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
