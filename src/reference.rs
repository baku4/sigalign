use crate::core::Sequence;
use crate::core::{ReferenceInterface, PatternLocation};

mod pattern_matching;
use pattern_matching::LtFmIndex;

// For test
mod test_reference;
pub use test_reference::TestReference;

use serde::{Deserialize, Serialize};

use std::marker::PhantomData;
use std::collections::HashMap;

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
    lt_fm_index: LtFmIndex,
    /// Accumulated lengths of records for locating k-sized pattern
    ///  - Length of vector is record count + 1
    ///  - First element must be 0
    accumulated_length: Vec<u64>,
}
impl PatternLocater {
    fn locate_in_search_range(&self, pattern: Sequence, search_range: &SearchRange) -> Vec<PatternLocation> {
        let sorted_locations = self.sorted_locations_of_pattern(pattern);

        let mut positions_by_record: HashMap<usize, Vec<usize>> = HashMap::new();
        // TODO: (1) Apply capacity (2) Change to faster hasher

        let pattern_size = pattern.len() as u64;
        let search_range_count = search_range.0.len();

        let mut size;
        let mut left;
        let mut right;
        let mut mid = 0;
        let mut index;

        for position in sorted_locations {
            // reset
            right = search_range_count;
            left = mid;
            size = right - left;
    
            while left < right {
                mid = left + size / 2;
                index = search_range.0[mid];
                
                let start = self.accumulated_length[index];
                let end = self.accumulated_length[index + 1];

                if position >= end {
                    left = mid + 1;
                } else if start > position {
                    right = mid;
                } else {
                    if (position + pattern_size) < end {
                        let ref_pos = (position - start) as usize;
                        match positions_by_record.get_mut(&index) {
                            Some(v) => {
                                v.push(ref_pos);
                            },
                            None => {
                                positions_by_record.insert(index, vec![ref_pos]);
                            },
                        }
                        break;
                    } else {
                        break;
                    }
                }
    
                size = right - left;
            }
        }
    
        positions_by_record.into_iter().map(|(record_index, positions)| {
            PatternLocation {
                record_index: record_index,
                positions: positions,
            }
        }).collect()
    }
    fn sorted_locations_of_pattern(&self, pattern: Sequence) -> Vec<u64> {
        let mut locations = self.lt_fm_index.locate(pattern);
        locations.sort();
        locations
    }
}

pub trait SequenceProvider<'a> {
    fn total_record_count(&self) -> usize;
    fn sequence_of_record(&self, record_index: usize) -> &'a [u8];
    fn joined_sequence_and_accumulated_lengths(&self) -> (Vec<u8>, Vec<u64>) {
        let total_record_count = self.total_record_count();
        let mut accumulated_lengths = Vec::with_capacity(total_record_count + 1);
        accumulated_lengths.push(0);

        let joined_sequence: Vec<u8> = (0..total_record_count).map(|record_index| {
            let record = self.sequence_of_record(record_index);
            accumulated_lengths.push(record.len() as u64);

            record
        }).flatten().map(|character| *character).collect();

        (joined_sequence, accumulated_lengths)
    }
}

struct ReferenceConfig {
    // For sequence type
    nucleotide: bool,
    with_noise: bool,
    // For lt-fm-index
    bwt_block_size_is_64: bool,
    sampling_ratio: usize,
    kmer_size_for_lookup_table: usize,
}
