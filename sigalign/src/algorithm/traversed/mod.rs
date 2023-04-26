use super::{AnchorTable, Anchor, AnchorIndex};

#[inline]
pub fn transform_left_additive_position_to_traversed_anchor_index(
    anchor_table: &AnchorTable,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    base_pattern_index: u32,
    base_target_position: u32,
    index_range: (u32, u32),
) {
    traversed_anchor_index_buffer[index_range.0 as usize..index_range.1 as usize].iter_mut().for_each(|additive_position_info| {
        let pattern_index = base_pattern_index - additive_position_info.0;
        let target_position = base_target_position - additive_position_info.1;
        let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
        let anchor_index_in_pattern = unsafe {
            binary_search(anchors_by_pattern, target_position).unwrap_unchecked()
        } as u32;
        *additive_position_info = (pattern_index, anchor_index_in_pattern);
    });
}
#[inline]
pub fn transform_right_additive_position_to_traversed_anchor_index(
    anchor_table: &AnchorTable,
    traversed_anchor_index_buffer: &mut Vec<AnchorIndex>,
    base_pattern_index: u32,
    base_target_position: u32,
    index_range: (u32, u32),
    pattern_size: u32,
) {
    traversed_anchor_index_buffer[index_range.0 as usize..index_range.1 as usize].iter_mut().for_each(|additive_position_info| {
        let mut pattern_index = base_pattern_index + additive_position_info.0;
        let mut target_position = base_target_position + additive_position_info.1;
        let anchor_index_in_pattern = {
            loop {
                let anchors_by_pattern = &anchor_table.0[pattern_index as usize];
                match binary_search(
                    anchors_by_pattern,
                    target_position,
                ) {
                    Ok(v) => {
                        break v as u32
                    },
                    Err(_) => {
                        pattern_index -= 1;
                        target_position -= pattern_size;
                    },
                }
            }
        };
        *additive_position_info = (pattern_index, anchor_index_in_pattern);
    });
}

#[inline(always)]
fn binary_search(
    anchors_by_pattern: &Vec<Anchor>,
    target_position: u32,
) -> Result<usize, usize> {
    anchors_by_pattern.binary_search_by_key(&target_position, |anchor| {
        anchor.target_position
    })
}
