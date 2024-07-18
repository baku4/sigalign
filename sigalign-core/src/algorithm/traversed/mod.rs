use super::{Anchor, AnchorTable, TraversedAnchor};

#[inline]
pub fn transform_right_additive_positions_to_traversed_anchor_index(
    anchor_table: &AnchorTable,
    traversed_anchors_buffer: &mut Vec<TraversedAnchor>,
    base_pattern_index: u32,
    base_target_position: u32,
    pattern_size: u32,
) {
    traversed_anchors_buffer.iter_mut().for_each(|tv| {
        let mut pattern_index = base_pattern_index + tv.addt_pattern_index;
        let mut target_position = base_target_position + tv.addt_target_position;
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
        tv.addt_pattern_index = pattern_index;
        tv.addt_target_position = anchor_index_in_pattern;
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
