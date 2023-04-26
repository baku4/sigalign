use super::{
    Anchor,
    AnchorTable,
    AnchorIndex,
};
use std::cmp::Ordering;

#[inline(always)]
pub fn mark_anchor_as_extended(
    anchor: &Anchor,
    extension_index: u32,
) {
    unsafe {
        std::ptr::write(&anchor.extended as *const bool as *mut bool, true);
        std::ptr::write(&anchor.extension_index as *const u32 as *mut u32, extension_index);
    };
}

#[inline]
pub fn mark_traversed_anchors_as_skipped(
    anchor_table: &AnchorTable,
    traversed_anchor_index_buffer: &Vec<AnchorIndex>,
    // of current anchor
    current_anchor_index: (u32, u32),
    right_traversed_anchor_index_start: u32,
    right_traversed_anchor_index_end: u32,
    // of right traversed anchor
    left_traversed_anchor_index_start: u32,
    left_traversed_anchor_index_end: u32,
) {
    // Init
    //   - Except right most anchor (= extended traversed anchor)
    let mut left_traversed_anchor_count = left_traversed_anchor_index_end - left_traversed_anchor_index_start;
    if left_traversed_anchor_count == 0 {
        return
    }
    let mut right_traversed_anchor_count = right_traversed_anchor_index_end - right_traversed_anchor_index_start - 1;
    
    // Skip middle range
    let mut right_anchor_index_ptr = unsafe {
        (&traversed_anchor_index_buffer[
            right_traversed_anchor_index_start as usize
        ] as *const AnchorIndex).add(1)
    };
    let mut left_anchor_index_ptr = &traversed_anchor_index_buffer[
        left_traversed_anchor_index_end as usize - 1
    ] as *const AnchorIndex;
    while right_traversed_anchor_count > 0 && left_traversed_anchor_count > 0 {
        match cmp_anchor_index(
            right_anchor_index_ptr,
            left_anchor_index_ptr,
        ) {
            Ordering::Less => { // right is lesser than left
                unsafe {
                    left_anchor_index_ptr = left_anchor_index_ptr.sub(1);
                };
                left_traversed_anchor_count -= 1;
            },
            Ordering::Equal => {
                mark_anchor_as_skipped(
                    anchor_table,
                    right_anchor_index_ptr,
                );
                unsafe {
                    right_anchor_index_ptr = right_anchor_index_ptr.add(1);
                    left_anchor_index_ptr = left_anchor_index_ptr.sub(1);
                };
                right_traversed_anchor_count -= 1;
                left_traversed_anchor_count -= 1;
            },
            Ordering::Greater => {
                unsafe {
                    right_anchor_index_ptr = right_anchor_index_ptr.add(1);
                };
                right_traversed_anchor_count -= 1;
            },
        }
    }

    // If the current index is contained in the remained left traversed anchors:
    // Mark the right anchors as skip
    for _ in 0..left_traversed_anchor_count {
        if cmp_anchor_index(
            &current_anchor_index as *const AnchorIndex,
            left_anchor_index_ptr,
        ) == Ordering::Equal {
            let rightmost_traversed_anchor_index_ptr = &traversed_anchor_index_buffer[
                right_traversed_anchor_index_start as usize
            ] as *const AnchorIndex;
            mark_anchor_as_skipped(
                anchor_table,
                rightmost_traversed_anchor_index_ptr,
            );
            break;
        } else {
            left_anchor_index_ptr = unsafe {
                left_anchor_index_ptr.sub(1)
            };
        }
    };
}

#[inline(always)]
fn cmp_anchor_index(
    right_anchor_index_ptr: *const AnchorIndex,
    left_anchor_index_ptr: *const AnchorIndex,
) -> Ordering {
    unsafe {
        let r = &*right_anchor_index_ptr;
        let l = &*left_anchor_index_ptr;
        let result =  r.0.cmp(&l.0);
        if result == Ordering::Equal {
            r.1.cmp(&l.1)
        } else {
            result
        }
    }
}

#[inline(always)]
fn mark_anchor_as_skipped(
    anchor_table: &AnchorTable,
    anchor_index_ptr: *const AnchorIndex,
) {
    unsafe {
        let (i1, i2) = *anchor_index_ptr;
        std::ptr::write(
            &anchor_table.0[i1 as usize][i2 as usize].skipped as *const bool as *mut bool,
            true,
        );
    }
}