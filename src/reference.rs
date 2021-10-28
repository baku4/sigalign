use crate::{Result, error_msg};
use crate::core::Sequence;
use crate::core::{ReferenceInterface, PatternLocation};

mod pattern_matching;
pub mod sequence_provider;
mod io;
#[cfg(test)]
mod test_reference;

use pattern_matching::LtFmIndex;
pub use sequence_provider::{InMemoryProvider, IndexedFastaProvider, SqliteProvider};
#[cfg(test)]
pub use test_reference::TestReference;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct Reference<S: SequenceProvider> {
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
    pub fn new(
        lt_fm_index_config: LtFmIndexConfig,
        mut sequence_provider: S
    ) -> Result<Self> {
        let total_record_count = sequence_provider.total_record_count();
        let search_range = (0..total_record_count).collect();

        let (joined_sequence, accumulated_lengths) = sequence_provider.joined_sequence_and_accumulated_lengths();

        let sequence_type = SequenceType::inferred_from_sequence(&joined_sequence)?;

        let pattern_locater = PatternLocater::new(
            &sequence_type,
            lt_fm_index_config,
            joined_sequence,
            accumulated_lengths
        );

        Ok(
            Self {
                sequence_type,
                total_record_count,
                search_range,
                pattern_locater,
                sequence_provider,
            }
        )
    }
    pub fn new_with_sequence_type(
        sequence_type: SequenceType,
        lt_fm_index_config: LtFmIndexConfig,
        mut sequence_provider: S
    ) -> Result<Self> {
        let total_record_count = sequence_provider.total_record_count();
        let search_range = (0..total_record_count).collect();

        let (joined_sequence, accumulated_lengths) = sequence_provider.joined_sequence_and_accumulated_lengths();

        if !sequence_type.is_searchable(&joined_sequence) {
            error_msg!("Sequence provider supply unsearchable sequence.");
        }

        let pattern_locater = PatternLocater::new(
            &sequence_type,
            lt_fm_index_config,
            joined_sequence,
            accumulated_lengths
        );

        Ok(
            Self {
                sequence_type,
                total_record_count,
                search_range,
                pattern_locater,
                sequence_provider,
            }
        )
    }
    pub fn new_unchecked(
        sequence_type: SequenceType,
        lt_fm_index_config: LtFmIndexConfig,
        mut sequence_provider: S
    ) -> Self {
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
    pub fn set_search_range(&mut self, mut search_range: Vec<usize>) -> Result<()> {
        search_range.sort();
        match search_range.last() {
            Some(&last_record_index) => {
                if last_record_index > self.total_record_count {
                    error_msg!("Search range is out of reference bound.")
                } else {
                    self.set_search_range_unchecked(search_range);
                    Ok(())
                }
            },
            None => error_msg!("Search range is empty.")
        }
    }
    pub fn set_search_range_unchecked(&mut self, search_range: Vec<usize>) {
        self.search_range = search_range;
    }
}

impl<SL: SequenceProvider + Labeling> Reference<SL> {
    pub fn label_of_record(&mut self, record_index: usize) -> &str {
        self.sequence_provider.label_of_record(record_index)
    }
}

const NUCLEOTIDE_UTF8: [u8; 4] = [65, 67, 71, 84]; // A, C, G, T
const AMINO_ACID_UTF8: [u8; 20] = [65, 67, 68, 69, 70, 71, 72, 73, 75, 76, 77, 78, 80, 81, 82, 83, 84, 86, 87, 89]; // A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceType {
    allowed_type: AllowedSequenceType,
    utf8_chr_of_type: Vec<u8>,
}
impl SequenceType {
    pub fn inferred_from_sequence(sequence: Sequence) -> Result<Self> {
        let mut presume_nucleotide = true;
        let mut noise_of_nucleotide = None;
        let mut noise_of_amino_acid = None;

        for &character in sequence {
            match character {
                65 | 67 | 71 | 84 => { // ACGT
                    // nothing to do
                },
                68 | 69 | 70 | 72 | 73 | 75 | 76 | 77 | 78
                | 80 | 81 | 82 | 83 | 86 | 87 | 89 => { // Non ACGT Aminoacid
                    if presume_nucleotide {
                        match noise_of_nucleotide {
                            Some(noise) => {
                                if noise != character {
                                    presume_nucleotide = false;
                                }
                            },
                            None => {
                                noise_of_nucleotide = Some(character);
                            },
                        }
                    }
                },
                _ => {
                    if presume_nucleotide {
                        match noise_of_nucleotide {
                            Some(_) => {},
                            None => {
                                noise_of_nucleotide = Some(character);
                            },
                        }
                    }
                    match noise_of_amino_acid {
                        Some(noise) => {
                            if noise != character {
                                error_msg!("Sequence is not supported type")
                            }
                        },
                        None => {
                            noise_of_amino_acid = Some(character);
                        },
                    }
                },
            }
        }

        Ok(
            if presume_nucleotide {
                match noise_of_nucleotide {
                    Some(noise) => {
                        Self::nucleotide_with_noise(noise)
                    },
                    None => {
                        Self::nucleotide_only()
                    },
                }
            } else {
                match noise_of_amino_acid {
                    Some(noise) => {
                        Self::aminoacid_with_noise(noise)
                    },
                    None => {
                        Self::aminoacid_only()
                    },
                }
            }
        )
    }
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
    pub fn allowed_type(&self) -> AllowedSequenceType {
        self.allowed_type.clone()
    }
    fn is_searchable(&self, query: Sequence) -> bool {
        query.iter().all(|character| {
            self.utf8_chr_of_type.contains(character)
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedSequenceType {
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
        let mut accumulated_length = 0;

        let joined_sequence: Vec<u8> = (0..total_record_count).map(|record_index| {
            let record = self.sequence_of_record(record_index).to_vec();
            accumulated_length += record.len() as u64;
            accumulated_lengths.push(accumulated_length);

            record
        }).flatten().collect();
        

        (joined_sequence, accumulated_lengths)
    }
}

pub trait Labeling {
    fn label_of_record(&mut self, record_index: usize) -> &str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_infer_sequence_type() {
        let nucleotide_only_sequence = b"ACGTACGT";
        let nucleotide_with_noise_sequence = b"ACGTNACGTN";
        let amino_acid_only_sequence = b"ACDEFGHIKLMNPQRSTVWYACDEFGHIKLMNPQRSTVWY";
        let amino_acid_with_noise_sequence = b"ACDEFGHIKLMNPQRSTVWYXACDEFGHIKLMNPQRSTVWYX";

        let errored_sequence = b"ACGTXZ";

        assert_eq!(
            SequenceType::inferred_from_sequence(nucleotide_only_sequence).unwrap(),
            SequenceType::nucleotide_only(),
        );

        assert_eq!(
            SequenceType::inferred_from_sequence(nucleotide_with_noise_sequence).unwrap(),
            SequenceType::nucleotide_with_noise(b'N'),
        );

        assert_eq!(
            SequenceType::inferred_from_sequence(amino_acid_only_sequence).unwrap(),
            SequenceType::aminoacid_only(),
        );

        assert_eq!(
            SequenceType::inferred_from_sequence(amino_acid_with_noise_sequence).unwrap(),
            SequenceType::aminoacid_with_noise(b'X'),
        );

        assert!(SequenceType::inferred_from_sequence(errored_sequence).is_err());
    }
}