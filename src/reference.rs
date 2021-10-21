use crate::core::Sequence;
use crate::core::{ReferenceInterface, PatternLocation};

mod pattern_matching;
mod sequence_provider;

use pattern_matching::LtFmIndex;

// For test
mod test_reference;
pub use test_reference::TestReference;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct Reference<S: SequenceProvider> {
    sequence_type: SequenceType,
    total_record_count: usize,
    search_range: Vec<usize>,
    pattern_locater: PatternLocater,
    sequence_provider: S,
}

impl<S: SequenceProvider> ReferenceInterface for Reference<S> {
    fn is_searchable(&self, query: Sequence) -> bool {
        self.sequence_type.is_searchable(query)
    }
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.pattern_locater.locate_in_search_range(pattern, &self.search_range)
    }
    fn sequence_of_record(&mut self, record_index: usize) -> Sequence {
        self.sequence_provider.sequence_of_record(record_index)
    }
}

impl<S: SequenceProvider> Reference<S> {
    fn new(sequence_type: SequenceType, lt_fm_index_config: LtFmIndexConfig, mut sequence_provider: S) -> Self {
        let total_record_count = sequence_provider.total_record_count();
        let search_range = (0..total_record_count).collect();

        let (joined_sequence, accumulated_lengths) = sequence_provider.joined_sequence_and_accumulated_lengths();

        let pattern_locater = PatternLocater::new(
            &sequence_type,
            lt_fm_index_config,
            joined_sequence,
            accumulated_lengths
        );

        Self {
            sequence_type,
            total_record_count,
            search_range,
            pattern_locater,
            sequence_provider,
        }
    }
    fn set_search_range(&mut self, mut search_range: Vec<usize>) {
        search_range.sort();
        self.set_search_range_unchecked(search_range);
    }
    fn set_search_range_unchecked(&mut self, search_range: Vec<usize>) {
        self.search_range = search_range;
    }
}

const NUCLEOTIDE_UTF8: [u8; 4] = [65, 67, 71, 84]; // A, C, G, T
const AMINO_ACID_UTF8: [u8; 20] = [65, 67, 68, 69, 70, 71, 72, 73, 75, 76, 77, 78, 80, 81, 82, 83, 84, 86, 87, 89]; // A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y

#[derive(Debug, Serialize, Deserialize)]
pub struct SequenceType {
    allowed_type: AllowedSequenceType,
    utf8_chr_of_type: Vec<u8>,
}
impl SequenceType {
    pub fn nucleotide_only() -> Self {
        Self {
            allowed_type: AllowedSequenceType::NucleotideOnly,
            utf8_chr_of_type: NUCLEOTIDE_UTF8.to_vec(),
        }
    }
    pub fn nucleotide_with_noise(character_for_noise: u8) -> Self {
        let mut utf8_chr_of_type = NUCLEOTIDE_UTF8.to_vec();
        utf8_chr_of_type.push(character_for_noise);
        Self {
            allowed_type: AllowedSequenceType::NucleotideWithNoise,
            utf8_chr_of_type,
        }
    }
    pub fn aminoacid_only() -> Self {
        Self {
            allowed_type: AllowedSequenceType::AminoacidOnly,
            utf8_chr_of_type: AMINO_ACID_UTF8.to_vec(),
        }
    }
    pub fn aminoacid_with_noise(character_for_noise: u8) -> Self {
        let mut utf8_chr_of_type = AMINO_ACID_UTF8.to_vec();
        utf8_chr_of_type.push(character_for_noise);
        Self {
            allowed_type: AllowedSequenceType::AminoacidWithNoise,
            utf8_chr_of_type,
        }
    }
    fn is_searchable(&self, query: Sequence) -> bool {
        query.iter().all(|character| {
            self.utf8_chr_of_type.contains(character)
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
enum AllowedSequenceType {
    NucleotideOnly, // NO
    NucleotideWithNoise, // NN
    AminoacidOnly, // AO
    AminoacidWithNoise, // AN
}

#[derive(Debug, Serialize, Deserialize)]
struct PatternLocater {
    lt_fm_index: LtFmIndex,
    /// Accumulated lengths of records for locating k-sized pattern
    ///  - Length of vector is record count + 1
    ///  - First element must be 0
    accumulated_lengths: Vec<u64>,
}
impl PatternLocater {
    fn new(
        sequence_type: &SequenceType,
        lt_fm_index_config: LtFmIndexConfig,
        joined_sequence: Vec<u8>,
        accumulated_lengths: Vec<u64>,
    ) -> Self {
        let (is_nucleotide, with_noise) = match sequence_type.allowed_type {
            AllowedSequenceType::NucleotideOnly => (true, false),
            AllowedSequenceType::NucleotideWithNoise => (true, true),
            AllowedSequenceType::AminoacidOnly => (false, false),
            AllowedSequenceType::AminoacidWithNoise => (false, true),
        };

        let lt_fm_index = LtFmIndex::new(
            is_nucleotide,
            with_noise,
            lt_fm_index_config.use_bwt_size_of_128,
            lt_fm_index_config.sa_sampling_ratio,
            lt_fm_index_config.kmer_size_for_lookup_table,
            joined_sequence,
        );

        Self {
            lt_fm_index,
            accumulated_lengths,
        }
    }
    fn locate_in_search_range(&self, pattern: Sequence, search_range: &Vec<usize>) -> Vec<PatternLocation> {
        let sorted_locations = self.sorted_locations_of_pattern(pattern);

        let mut positions_by_record: HashMap<usize, Vec<usize>> = HashMap::new();
        // TODO: (1) Apply capacity (2) Change to faster hasher

        let pattern_size = pattern.len() as u64;
        let search_range_count = search_range.len();

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
                index = search_range[mid];
                
                let start = self.accumulated_lengths[index];
                let end = self.accumulated_lengths[index + 1];

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

pub struct LtFmIndexConfig {
    use_bwt_size_of_128: bool,
    sa_sampling_ratio: u64,
    kmer_size_for_lookup_table: Option<usize>, // Use default if not specified
}

impl Default for LtFmIndexConfig {
    fn default() -> Self {
        Self {
            use_bwt_size_of_128: false,
            sa_sampling_ratio: 2,
            kmer_size_for_lookup_table: None,
        }
    }
}

impl LtFmIndexConfig {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn use_bwt_size_of_128(mut self) -> Self {
        self.use_bwt_size_of_128 = true;
        self
    }
    pub fn change_sampling_ratio(mut self, sa_sampling_ratio: u64) -> Self {
        self.sa_sampling_ratio = sa_sampling_ratio;
        self
    }
    pub fn change_kmer_size_for_lookup_table(mut self, kmer_size: usize) -> Self {
        self.kmer_size_for_lookup_table = Some(kmer_size);
        self
    }
}

pub trait SequenceProvider {
    fn total_record_count(&self) -> usize;
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8];
    fn joined_sequence_and_accumulated_lengths(&mut self) -> (Vec<u8>, Vec<u64>) {
        let total_record_count = self.total_record_count();
        let mut accumulated_lengths = Vec::with_capacity(total_record_count + 1);
        accumulated_lengths.push(0);

        let joined_sequence: Vec<u8> = (0..total_record_count).map(|record_index| {
            let record = self.sequence_of_record(record_index).to_vec();
            accumulated_lengths.push(record.len() as u64);

            record
        }).flatten().collect();

        (joined_sequence, accumulated_lengths)
    }
}
