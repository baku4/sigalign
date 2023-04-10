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
use super::{PosTable, AnchorIndex, AnchorPosition, TraversedAnchorDep};
use super::{Extension, WaveFront, WaveFrontScore, BackTraceMarker, calculate_spare_penalty};
use super::{
    SideExtension,
};
use std::cmp::Reverse;
use ahash::AHashSet;

#[inline]
pub fn sort_left_side_extensions(
    side_extensions: &mut Vec<SideExtension>,
) {
    side_extensions.sort_unstable_by_key(|side_extension| {
        side_extension.last_query_index
    });
}
#[inline]
pub fn sort_right_side_extensions(
    side_extensions: &mut Vec<SideExtension>,
) {
    side_extensions.sort_unstable_by_key(|side_extension| {
        Reverse(side_extension.last_query_index)
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