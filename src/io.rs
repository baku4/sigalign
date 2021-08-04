pub mod cigar;

use crate::{SequenceLength, Penalty};

#[derive(Debug)]
pub struct Alignment {
    pub penalty: Penalty,
    pub length: SequenceLength,
    pub clip_front: cigar::Clip,
    pub clip_end: cigar::Clip,
    pub cigar: cigar::Cigar,
}