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
    total_record_count: usize,
    search_range: SearchRange,
    pattern_locater: PatternLocater,
    sequence_provider: S,
    phantom_data: PhantomData<&'a S>,
}

impl<'a, S> ReferenceInterface for Reference<'a, S> where S: SequenceProvider<'a> {
    fn is_searchable(&self, query: Sequence) -> bool {
        self.sequence_type.is_searchable(query)
    }
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.pattern_locater.locate_in_search_range(pattern)
    }
    fn sequence_of_record(&self, record_index: usize) -> Sequence {
        self.sequence_provider.sequence_of_record(record_index)
    }
}

const NucleotideUTF8: [u8; 4] = [65, 67, 71, 84]; // A, C, G, T
const AminoAcidUTF8: [u8; 20] = [65, 67, 68, 69, 70, 71, 72, 73, 75, 76, 77, 78, 80, 81, 82, 83, 84, 86, 87, 89]; // A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y

#[derive(Debug, Serialize, Deserialize)]
struct SequenceType {
    type_marker: SequenceTypeMarker,
    allowed_utf8: Vec<u8>,
}
impl SequenceType {
    fn is_searchable(&self, query: Sequence) -> bool {
        query.iter().all(|character| {
            self.allowed_utf8.contains(character)
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum SequenceTypeMarker {
    NucleotideOnly, // NO
    NucleotideWithNoise, // NN
    AminoAcidOnly, // AO
    AminoAcidWithNoise, // AN
}

#[derive(Debug, Serialize, Deserialize)]
struct SearchRange(Vec<usize>);

#[derive(Debug, Serialize, Deserialize)]
struct PatternLocater {
    fm_index: FmIndex,
    accumulated_length: AccumulatedLength,
}
impl PatternLocater {
    fn locate_in_search_range(&self, pattern: Sequence) -> Vec<PatternLocation> {
        
    }
}

/// Accumulated length for locating k-sized pattern
/// (start, end)
pub type AccumulatedLength = Vec<(u64, u64)>;

pub trait SequenceProvider<'a> {
    fn sequence_of_record(&self, record_index: usize) -> &'a [u8];
    fn joined_sequence(&self) -> Vec<u8>;
    fn accumulated_length(&self) -> AccumulatedLength;
}
