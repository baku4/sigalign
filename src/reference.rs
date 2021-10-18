use crate::core::Sequence;
use crate::core::{ReferenceInterface, PatternLocation};

mod pattern_matching;
use pattern_matching::FmIndex;

// For test
mod test_reference;
pub use test_reference::TestReference;

use serde::{Deserialize, Serialize};

use std::marker::PhantomData;

#[derive(Debug, Serialize, Deserialize)]
struct Reference<'a, S> where S: SequenceProvider<'a> {
    sequence_type: SequenceType,
    pattern_locater: PatternLocater,
    sequence_provider: S,
    phantom_data: PhantomData<&'a S>,
}

impl<'a, S> ReferenceInterface for Reference<'a, S> where S: SequenceProvider<'a> {
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.pattern_locater.locate(pattern)
    }
    fn sequence_of_record(&self, record_index: usize) -> Sequence {
        self.sequence_provider.sequence_of_record(record_index)
    }
    fn is_searchable(&self, pattern: Sequence) -> bool {
        self.sequence_type.is_searchable(pattern)
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SequenceType {
    NucleotideOnly, // NO
    NucleotideWithNoise, // NN
    AminoAcidOnly, // AO
    AminoAcidWithNoise, // AN
}

impl SequenceType {
    fn is_searchable(&self, pattern: Sequence) -> bool {
        
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PatternLocater {
    fm_index: FmIndex,
    search_range: SearchRange,
    accumulated_length: AccumulatedLength,
}
impl PatternLocater {
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {

    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchRange(Vec<usize>);

/// Accumulated length for locating k-sized pattern
/// (start, end)
pub type AccumulatedLength = Vec<(u64, u64)>;

pub trait SequenceProvider<'a> {
    fn sequence_of_record(&self, record_index: usize) -> &'a [u8];
    fn joined_sequence(&self) -> Vec<u8>;
    fn accumulated_length(&self) -> AccumulatedLength;
}
