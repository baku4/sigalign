use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
    Reference, SequenceProvider,
};

use super::{PosTable, AnchorPosition, AnchorIndex, Extension};

#[derive(Debug, Clone)]
pub struct Traversed {
    pub sorted_list: Vec<AnchorIndex>,
    pub at_the_end: Option<AnchorIndex>,
}

impl PosTable {
    pub fn right_traversed_anchors(&self, extension: &Extension, start_pattern_index: usize) {
        //TODO:
    }
}

impl AnchorPosition {
    fn binary_search_index(pattern_position: &Vec<Self>, record_position: &usize) -> usize {
        pattern_position.binary_search_by_key(record_position, |anchor_position| {
            anchor_position.record_position
        }).unwrap()
    }
}
