mod extending;
mod evaluating;

pub use extending::{DropoffWaveFront, WaveFrontScore, Components, Component};
pub use extending::{M_COMPONENT, I_COMPONENT, D_COMPONENT, EMPTY, FROM_M, FROM_I, FROM_D, START};
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

pub struct Cutoff {
    pub minimum_aligned_length: usize,
    pub penalty_per_length: f32,
}

pub struct MinPenaltyForPattern {
    pub odd: usize,
    pub even: usize,
}


// TEXT


pub type Sequence<'a> = &'a [u8];

pub trait ReferenceInterface {
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation>;
    fn sequence_of_record(&self, record_index: usize) -> Sequence;
    fn is_searchable(&self, pattern: Sequence) -> bool;
}

pub struct PatternLocation {
    pub record_index: usize,
    pub positions: Vec<usize>,
}


// RESULTS


pub type AlignmentResultsByRecord = HashMap<usize, Vec<AlignmentResult>>;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlignmentOperation {
    pub alignment_type: AlignmentType,
    pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlignmentType {
    Match,
    Subst,
    Insertion,
    Deletion,
}


// ALGORITHM


pub trait Algorithm {
    fn alignment(
        reference: &dyn ReferenceInterface,
        query: Sequence,
        pattern_size: usize,
        penalties: &Penalties,
        cutoff: &Cutoff,
        min_penalty_for_pattern: &MinPenaltyForPattern,
    ) -> AlignmentResultsByRecord;
}
