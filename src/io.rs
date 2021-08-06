pub mod cigar;
pub mod reader;

use crate::{SequenceLength, Penalty};

// DATA STRUCTURES

#[derive(Debug)]
pub struct Alignment {
    pub penalty: Penalty,
    pub length: SequenceLength,
    pub clip_front: cigar::Clip,
    pub clip_end: cigar::Clip,
    pub cigar: cigar::Cigar,
}