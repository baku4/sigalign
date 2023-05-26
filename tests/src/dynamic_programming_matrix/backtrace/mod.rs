use super::{DpMatrix, Cell, BacktraceMarker};
use sigalign::results::{
    AlignmentOperations,
    AlignmentOperation, AnchorAlignmentResult, AlignmentPosition,
};
use std::cmp;
use ahash::AHashSet;

mod common;
use common::{
    parse_the_unique_alignments_and_its_path,
    concat_ops,
    get_alignment_position,
};
mod local;
use local::parse_valid_local_result;
mod semi_global;
use semi_global::parse_valid_semi_global_result;

impl DpMatrix {
    pub fn parse_valid_local_result(
        &self,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Vec<AnchorAlignmentResult> {
        parse_valid_local_result(self, minimum_length, maximum_penalty_per_length)
    }
    pub fn parse_valid_semi_global_result(
        &self,
        minimum_length: u32,
        maximum_penalty_per_length: f32,
    ) -> Vec<AnchorAlignmentResult> {
        parse_valid_semi_global_result(self, minimum_length, maximum_penalty_per_length)
    }
}
