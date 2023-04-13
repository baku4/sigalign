use crate::{
    core::{
        SeqLen,
        regulators::{
            Penalty, PREC_SCALE, Cutoff,
        },
    },
    results::{
        AlignmentOperation, AnchorAlignmentResult, AlignmentPosition, AlignmentOperations,
    }
};
use super::{
    AnchorTable, Anchor, AnchorIndex,
    MERGED_EMPTY_SYMBOL,
};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use super::{
    SideExtension,
    NewSideExtension,
    TraversedAnchor,
};
use std::cmp::Reverse;
use ahash::AHashSet;

pub struct PreAnchorAlignmentResult {
    pub left_traversed_anchor_index: u32,
    pub right_traversed_anchor_index: u32,
    pub query_start_index: u32,
    pub query_end_index: u32,
    pub symbol_start_index: u32,
}

#[inline]
pub fn evaluate(
    left_sorted_side_extensions: &mut Vec<NewSideExtension>,
    right_sorted_side_extensions: &mut Vec<NewSideExtension>,
    traversed_anchors_buffer: &Vec<TraversedAnchor>,
    symbols_buffer: &mut Vec<i32>,
    symbol_length: u32,
) {
    let minimum_length = 0;

    let mut pre_anchor_alignment_result_buffer: Vec<PreAnchorAlignmentResult> = Vec::new();
    let mut symbol_start_index_for_pre_result = symbols_buffer.len() as u32;
    // TODO: Optimize
    for _ in 0..symbol_length {
        symbols_buffer.push(MERGED_EMPTY_SYMBOL);
    }

    'outer: for left_side_extension in left_sorted_side_extensions.iter_mut() {
        let mut valid_result_of_left_is_searched = false;
        // (1) Compare left side extension with each right side extension
        'inner: for right_side_extension in right_sorted_side_extensions.iter_mut() {
            // Check if the loop can be escaped
            if left_side_extension.query_index_of_the_end >= right_side_extension.query_index_of_the_end {
                break 'outer;
            }
            if right_side_extension.skip_evaluation {
                continue 'inner;
            }

            // Compare symbol
            let mut left_symbol_ptr: *const i32 = &symbols_buffer[left_side_extension.symbol_start_index as usize];
            let mut right_symbol_ptr: *const i32 = &symbols_buffer[right_side_extension.symbol_start_index as usize];
            let mut merged_symbol_ptr: *const i32 = &symbols_buffer[symbol_start_index_for_pre_result as usize];
            // If the valid result is already searched:
            //   -> can skip other right side extensions
            if !valid_result_of_left_is_searched {
                let mut left_traversed_anchor_index: u32 = 0;
                let mut right_traversed_anchor_index: u32 = 0;
                for _ in 0..symbol_length {
                    let left_symbol = unsafe { *left_symbol_ptr };
                    let right_symbol = unsafe { *right_symbol_ptr };

                    // Have common traversed anchor
                    if left_symbol == right_symbol {
                        // Register the the common symbol
                        unsafe {
                            *(merged_symbol_ptr as *mut i32) = left_symbol;
                        }
                        if !valid_result_of_left_is_searched {
                            let left_traversed_anchor = &traversed_anchors_buffer[left_traversed_anchor_index as usize];
                            let right_traversed_anchor = &traversed_anchors_buffer[right_traversed_anchor_index as usize];

                            // Check if valid
                            let length = left_traversed_anchor.length + right_traversed_anchor.length;
                            let penalty_delta = left_traversed_anchor.penalty_delta + right_traversed_anchor.penalty_delta;
                            //   - (1) Check if the cutoff is satisfied 
                            if length >= minimum_length && penalty_delta >= 0 {
                                //   - (2) Check if the cutoff is satisfied
                                let pre_anchor_alignment_result = PreAnchorAlignmentResult {
                                    left_traversed_anchor_index,
                                    right_traversed_anchor_index,
                                    query_start_index: left_side_extension.query_index_of_the_end,
                                    query_end_index: right_side_extension.query_index_of_the_end,
                                    symbol_start_index: symbol_start_index_for_pre_result,
                                };
                                pre_anchor_alignment_result_buffer.push(pre_anchor_alignment_result);
                                valid_result_of_left_is_searched = true;
                            }
                        }
                    }
                    if left_symbol > 0 {
                        left_traversed_anchor_index += 1;
                    }
                    if right_symbol > 0 {
                        right_traversed_anchor_index += 1;
                    }
                    left_symbol_ptr = unsafe { left_symbol_ptr.add(1) };
                    right_symbol_ptr = unsafe { right_symbol_ptr.add(1) };
                    merged_symbol_ptr = unsafe { merged_symbol_ptr.add(1) };
                }

                // Start with "valid_result_of_left_is_searched" is false
                if valid_result_of_left_is_searched {
                    // If valid result is searched:
                    // Adjust the values for the next merged symbol
                    // TODO: Optimize
                    for _ in 0..symbol_length {
                        symbols_buffer.push(MERGED_EMPTY_SYMBOL);
                    }
                    symbol_start_index_for_pre_result = symbols_buffer.len() as u32;
                    merged_symbol_ptr = &symbols_buffer[symbol_start_index_for_pre_result as usize];
                } else {
                    // If there is no valid result:
                    // Rewind the values for merged symbol
                    for _ in 0..symbol_length {
                        merged_symbol_ptr = unsafe { merged_symbol_ptr.sub(1) };
                        unsafe {
                            *(merged_symbol_ptr as *mut i32) = MERGED_EMPTY_SYMBOL;
                        }
                    }
                }
            } else {
                for _ in 0..symbol_length {
                    let left_symbol = unsafe { *left_symbol_ptr };
                    let right_symbol = unsafe { *right_symbol_ptr };
                    if left_symbol == right_symbol {
                        right_side_extension.skip_evaluation = true;
                        continue 'inner;
                    }
                    left_symbol_ptr = unsafe { left_symbol_ptr.add(1) };
                    right_symbol_ptr = unsafe { right_symbol_ptr.add(1) };
                }
            }
        }
    }
}

#[inline]
fn get_merged_symbol(
    left_side_extension: &NewSideExtension,
    right_side_extension: &NewSideExtension,
) {
    
}

#[inline]
pub fn sort_left_side_extensions(
    side_extensions: &mut Vec<SideExtension>,
) {
    side_extensions.sort_unstable_by_key(|side_extension| {
        side_extension.query_index_of_the_end
    });
}
#[inline]
pub fn sort_right_side_extensions(
    side_extensions: &mut Vec<SideExtension>,
) {
    side_extensions.sort_unstable_by_key(|side_extension| {
        Reverse(side_extension.query_index_of_the_end)
    });
}

#[inline]
pub fn get_valid_extensions(
    left_side_extensions: &Vec<SideExtension>,
    right_side_extensions: &Vec<SideExtension>,
) {
    left_side_extensions.iter().for_each(|left_side_extension| {
        //
    });
}

#[inline(always)]
fn concatenate_extensions(
    left_side_extension: &SideExtension,
    right_side_extension: &SideExtension,
) {
    //
}