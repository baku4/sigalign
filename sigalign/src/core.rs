mod extending;
mod evaluating;

pub use extending::{Extension, WaveFront, EndPoint, WaveFrontScore, Components, Component, MatchBt, InsBt, DelBt};
pub use evaluating::AlignmentHashSet;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;


// CONDITIONS


#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Penalties {
    pub x: usize,
    pub o: usize,
    pub e: usize,
}

pub const PRECISION_SCALE: usize = 10_000; // Ensuring accuracy to the third decimal place.

#[derive(Debug)]
pub struct Cutoff {
    pub minimum_aligned_length: usize,
    pub maximum_penalty_per_scale: usize,
}

#[derive(Debug)]
pub struct MinPenaltyForPattern {
    pub odd: usize,
    pub even: usize,
}


// TEXT


pub type Sequence<'a> = &'a [u8];

pub trait ReferenceInterface {
    fn is_searchable(&self, query: Sequence) -> bool;
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation>;
    fn sequence_of_record(&mut self, record_index: usize) -> Sequence;
}

#[derive(Debug)]
pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}


// RESULTS


#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentResultsByRecordIndex(pub HashMap<usize, Vec<AlignmentResult>>);

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentResultsWithLabelByRecordIndex(
    pub HashMap<usize, (String, Vec<AlignmentResult>)>
);

#[derive(Debug, Serialize, Deserialize)]
pub struct AlignmentResult {
    pub dissimilarity: f32,
    pub penalty: usize,
    pub length: usize,
    pub position: AlignmentPosition,
    pub operations: Vec<AlignmentOperation>,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentPosition {
    pub record: (usize, usize),
    pub query: (usize, usize),
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct AlignmentOperation {
    pub alignment_type: AlignmentType,
    pub count: u32,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum AlignmentType {
    Match,
    Subst,
    Insertion,
    Deletion,
}


// ALGORITHM


pub trait Algorithm {
    fn alignment(
        reference: &mut dyn ReferenceInterface,
        query: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
        // TODO: Resolve dependency
        primary_wave_front: &mut WaveFront,
        secondary_wave_front: &mut WaveFront,
    ) -> AlignmentResultsByRecordIndex;
}
