/*! Configurations for [Reference] generation

`Reference` is a bundle of alignment target sequences.

# (1) Parameters to generate `Reference`
1. [SequenceType]
    - Definition of type of Sequence
        - **Four** types are supported.
            - Nucleotide only
                - A, C, G, T
            - Nucleotide with noise
                - A, C, G, T + One unspecified character
            - Aminoacid only
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y
            - Aminoacid with noise
                - A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y + One unspecified character
    - `SequenceType` is optional parameter for `Reference` generation because it can be inferred automatically. If passed, however, the `Reference` generation is faster because the inferring process can be skipped.
2. [LtFmIndexConfig]
    - Configurations for `fm-index` data structure of [`lt-fm-index` crate](https://crates.io/crates/lt-fm-index).
    - There are **three** options.
        - BWT size
            - Whether the size of the BWT block is 64 or 128.
            - Default is using 64 sized block.
            - Using 128 sized block lowers the memory usage of the `lt-fm-index` but slows the algorithm.
        - Sampling ratio
            - Sampling ratio for suffix array in `lt-fm-index`.
            - Default is 2.
            - The larger the value, the lower the memory usage, but the slower the algorithm.
        - Kmer size for lookup table
            - Kmer size for lookup table in `lt-fm-index`.
                - Lookup table is pre-calculated count array.
                - Using k-sized lookup table can skip first k LF-mapping operations.
            - Default value is depending on `lt-fm-index` crate.
                - [Configuration for `lt-fm-index`](https://docs.rs/lt-fm-index/0.4.0/lt_fm_index/struct.LtFmIndexConfig.html)
            - The larger the value, the larger the memory usage, but the faster the algorithm.
            - Since memory usage increases exponentially with the number of characters supported by the sequence type, it is not recommended to increase this value too much.
    - `LtFmIndexConfig` is optional parameter for `Reference` generation. If `LtFmIndexConfig` is not passed, the default config is used.
3. [SequenceProvider]
    - `SequenceProvider` is `trait` to provide sequence to `Reference`.
    - It require **two** methods.
        - `total_record_count` to get the number of records.
        - `sequence_of_record` to get sequence from index of record.
    - Method of `joined_sequence_and_accumulated_lengths` can be overrode for better performance.
        - `joined_sequence_and_accumulated_lengths` supply two vectors in tuple.
            - The "joined_sequence" means the sequence of concatenated sequences of all record.
            - The "accumulated_lengths" means the accumulated sequence lengths from 0 to the sum of the lengths of all sequences.
            - For examples, if there are three records with "ATT", "CC", "GGGG", the "joined_sequence" is "ATTCCGGGG" and the "accumulated_lengths" is [0, 3, 5, 9].
        - Since this "joined_sequence" can be very large in size (because there is all sequence), the strategy of memory allocation can be different for each sequence provider. For example, if the length of the entire sequence can be known in advance, allocating whole memory at once is faster than summing up each sequence.
        - Therefore, according to the size of the entire sequence and the characteristics of the sequence provider, whether to override this method or not can be determined by user.
    - `SequenceProvider` is mutable inside the `Reference`.
        - A buffer or pointer may be required.

# (2) Search range
- The search range is a list(`Vec` in Rust) of indexes of records to be searched.
- Search range can be set after `Reference` generation.
- When a reference is generated, it is set for the entire record (0..the number of records).
*/

use crate::{Result, error_msg};
use crate::core::Sequence;
use crate::core::{ReferenceInterface, PatternLocation};
use crate::utils::{FastaReader, reverse_complement_of_nucleotide_sequence};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

mod pattern_matching;
/// Basic implementations for sequence provider
pub mod basic_sequence_provider;
mod io;

use pattern_matching::LtFmIndex;
pub use io::Writable;

#[cfg(test)]
mod test_reference;
#[cfg(test)]
pub use test_reference::TestReference;

/// Alignment target [Reference]
#[derive(Debug, Serialize, Deserialize)]
pub struct Reference<S: SequenceProvider> {
    reference_proto: ReferenceProto,
    sequence_provider: S,
}
// For Serialize and Deserialize Reference separately from sequence provider
// Just to hold the data and easily serde
#[derive(Debug, Serialize, Deserialize)]
struct ReferenceProto {
    sequence_type: SequenceType,
    total_record_count: usize,
    search_range: Vec<usize>,
    pattern_locater: PatternLocater,
}
impl Writable for ReferenceProto {}

impl<S: SequenceProvider> ReferenceInterface for Reference<S> {
    fn searchable(&self, query: Sequence) -> bool {
        self.reference_proto.sequence_type.is_searchable(query)
    }
    fn locate(&self, pattern: Sequence) -> Vec<PatternLocation> {
        self.reference_proto.pattern_locater.locate_in_search_range(pattern, &self.reference_proto.search_range)
    }
    fn sequence_of_record(&mut self, record_index: usize) -> Sequence {
        self.sequence_provider.sequence_of_record(record_index)
    }
}

impl<S: SequenceProvider> Reference<S> {
    /// Generate new Reference with sequence provider.  
    /// - SequenceType is inferred.  
    /// - Default configuration for lt-fm-index is used.
    pub fn new_with_default_config(mut sequence_provider: S) -> Result<Self> {
        let total_record_count = sequence_provider.total_record_count();
        let search_range = (0..total_record_count).collect();

        let (joined_sequence, accumulated_lengths) = sequence_provider.joined_sequence_and_accumulated_lengths();

        let sequence_type = SequenceType::inferred_from_sequence(&joined_sequence)?;
        let lt_fm_index_config = LtFmIndexConfig::new();

        let pattern_locater = PatternLocater::new(
            &sequence_type,
            lt_fm_index_config,
            joined_sequence,
            accumulated_lengths
        );

        Ok(
            Self {
                reference_proto: ReferenceProto {
                    sequence_type,
                    total_record_count,
                    search_range,
                    pattern_locater,
                },
                sequence_provider,
            }
        )
    }
    /// Generate new Reference with lt-fm-index configuration and sequence provider.   
    /// - SequenceType is inferred.
    pub fn new_with_lt_fm_index_config(
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
                reference_proto: ReferenceProto {
                    sequence_type,
                    total_record_count,
                    search_range,
                    pattern_locater,
                },
                sequence_provider,
            }
        )
    }
    /// Generate new Reference with all custom configuration.
    pub fn new_with_config(
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
                reference_proto: ReferenceProto {
                    sequence_type,
                    total_record_count,
                    search_range,
                    pattern_locater,
                },
                sequence_provider,
            }
        )
    }
    /// Generate new Reference with all custom configuration.  
    /// - It does not check whether the `joined_sequence` is a supported `SequenceType`.
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
            reference_proto: ReferenceProto {
                sequence_type,
                total_record_count,
                search_range,
                pattern_locater,
            },
            sequence_provider,
        }
    }
    /// Set the search range, which is the set of indices of records to be found.
    pub fn set_search_range(&mut self, mut search_range: Vec<usize>) -> Result<()> {
        search_range.sort();
        match search_range.last() {
            Some(&last_record_index) => {
                if last_record_index > self.reference_proto.total_record_count {
                    error_msg!("Search range is out of reference bound.")
                } else {
                    self.set_search_range_unchecked(search_range);
                    Ok(())
                }
            },
            None => error_msg!("Search range is empty.")
        }
    }
    /// Set the search range.  
    /// - Do not check whether the search range does not exceed the total record count.  
    /// - Do not check if the search range is sorted.
    pub fn set_search_range_unchecked(&mut self, search_range: Vec<usize>) {
        self.reference_proto.search_range = search_range;
    }

    // Methods to get information of reference
    /// Get information of search range
    pub fn get_search_range(&self) -> &[usize] {
        &self.reference_proto.search_range
    }
    /// Get information of sequence type:
    /// (is nucleotide sequence, noise character)
    pub fn get_sequence_type(&self) -> (bool, Option<u8>) {
        self.reference_proto.sequence_type.is_nucleotide_with_noise_character()
    }
    /// Get information of suffix array sampling ratio in lt-fm-index
    pub fn get_sa_sampling_ratio(&self) -> u64 {
        self.reference_proto.pattern_locater.sa_sampling_ratio()
    }
    /// Get information of kmer size for lookup table in lt-fm-index
    pub fn get_kmer_size_for_lookup_table(&self) -> usize {
        self.reference_proto.pattern_locater.kmer_size_for_lookup_table()
    }
    /// Get information of Bwt block size in lt-fm-index
    pub fn get_size_of_bwt_block(&self) -> usize {
        self.reference_proto.pattern_locater.size_of_bwt_block()
    }
}

impl<SL: SequenceProvider + Labeling> Reference<SL> {
    pub fn label_of_record(&mut self, record_index: usize) -> &str {
        self.sequence_provider.label_of_record(record_index)
    }
}

const NUCLEOTIDE_UTF8: [u8; 4] = [65, 67, 71, 84]; // A, C, G, T
const AMINO_ACID_UTF8: [u8; 20] = [65, 67, 68, 69, 70, 71, 72, 73, 75, 76, 77, 78, 80, 81, 82, 83, 84, 86, 87, 89]; // A, C, D, E, F, G, H, I, K, L, M, N, P, Q, R, S, T, V, W, Y

/// Type of sequence
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SequenceType {
    allowed_type: AllowedSequenceType,
    utf8_chr_of_type: Vec<u8>,
}
impl SequenceType {
    /// Make new sequence type for nucleotide only
    pub fn nucleotide_only() -> Self {
        Self {
            allowed_type: AllowedSequenceType::NucleotideOnly,
            utf8_chr_of_type: NUCLEOTIDE_UTF8.to_vec(),
        }
    }
    /// Make new sequence type for nucleotide with noise
    pub fn nucleotide_with_noise(character_for_noise: u8) -> Self {
        let mut utf8_chr_of_type = NUCLEOTIDE_UTF8.to_vec();
        utf8_chr_of_type.push(character_for_noise);
        Self {
            allowed_type: AllowedSequenceType::NucleotideWithNoise,
            utf8_chr_of_type,
        }
    }
    /// Make new sequence type for amino acid only
    pub fn aminoacid_only() -> Self {
        Self {
            allowed_type: AllowedSequenceType::AminoacidOnly,
            utf8_chr_of_type: AMINO_ACID_UTF8.to_vec(),
        }
    }
    /// Make new sequence type for amino acid with noise
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
    pub fn is_nucleotide_with_noise_character(&self) -> (bool, Option<u8>) {
        let (is_nucleotide, has_noise) = self.allowed_type.is_nucleotide_and_has_noise();

        let noise_character = if has_noise {
            Some(*self.utf8_chr_of_type.last().unwrap())
        } else {
            None
        };

        (is_nucleotide, noise_character)
    }
    fn is_searchable(&self, query: Sequence) -> bool {
        query.iter().all(|character| {
            self.utf8_chr_of_type.contains(character)
        })
    }
    fn inferred_from_sequence(sequence: Sequence) -> Result<Self> {
        let mut can_be_nucleotide = true;
        let mut can_be_amino_acid = true;
        let mut noise_of_nucleotide = None;
        let mut noise_of_amino_acid = None;

        for &character in sequence {
            match character {
                65 | 67 | 71 | 84 => { // ACGT
                    // nothing to do
                },
                68 | 69 | 70 | 72 | 73 | 75 | 76 | 77 | 78
                | 80 | 81 | 82 | 83 | 86 | 87 | 89 => { // Non ACGT Aminoacid
                    if can_be_nucleotide {
                        match noise_of_nucleotide {
                            Some(noise) => {
                                if noise != character {
                                    can_be_nucleotide = false;
                                }
                            },
                            None => {
                                noise_of_nucleotide = Some(character);
                            },
                        }
                    }
                },
                _ => {
                    if can_be_nucleotide {
                        match noise_of_nucleotide {
                            Some(noise) => {
                                if noise != character {
                                    can_be_nucleotide = false;
                                }
                            },
                            None => {
                                noise_of_nucleotide = Some(character);
                            },
                        }
                    }
                    if can_be_amino_acid {
                        match noise_of_amino_acid {
                            Some(noise) => {
                                if noise != character {
                                    can_be_amino_acid = false;
                                }
                            },
                            None => {
                                noise_of_amino_acid = Some(character);
                            },
                        }
                    }
                    if !can_be_amino_acid && !can_be_nucleotide {
                        error_msg!("Sequence type is not supported")
                    }
                },
            }
        }

        Ok(
            if can_be_nucleotide {
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
}

/// Allowed four sequence types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AllowedSequenceType {
    NucleotideOnly, // NO
    NucleotideWithNoise, // NN
    AminoacidOnly, // AO
    AminoacidWithNoise, // AN
}
impl AllowedSequenceType {
    fn is_nucleotide_and_has_noise(&self) -> (bool, bool) {
        match self {
            Self::NucleotideOnly => (true, false),
            Self::NucleotideWithNoise => (true, true),
            Self::AminoacidOnly => (false, false),
            Self::AminoacidWithNoise => (false, true),
        }
    }
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
    // Kmer size of lt-fm-index
    fn kmer_size_for_lookup_table(&self) -> usize {
        self.lt_fm_index.kmer_size()
    }
    // Sampling ratio of lt-fm-index
    fn sa_sampling_ratio(&self) -> u64 {
        self.lt_fm_index.sa_sampling_ratio()
    }
    // Bwt block size
    fn size_of_bwt_block(&self) -> usize {
        self.lt_fm_index.bwt_size()
    }
}


/// Configuration for `lt-fm-index`
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
    /// New default configuration
    pub fn new() -> Self {
        Self::default()
    }
    /// Change the BWT block size from **64** to **128**.
    pub fn use_bwt_size_of_128(mut self) -> Self {
        self.use_bwt_size_of_128 = true;
        self
    }
    /// Change sampling ratio for suffix array
    pub fn change_sampling_ratio(mut self, sa_sampling_ratio: u64) -> Self {
        self.sa_sampling_ratio = sa_sampling_ratio;
        self
    }
    /// Change kmer size for lookup table
    pub fn change_kmer_size_for_lookup_table(mut self, kmer_size: usize) -> Self {
        self.kmer_size_for_lookup_table = Some(kmer_size);
        self
    }
}

/// Trait to provide sequence
pub trait SequenceProvider {
    /// The number of records
    fn total_record_count(&self) -> usize;
    /// Provide sequence stored in the index of record.
    fn sequence_of_record(&mut self, record_index: usize) -> &[u8];
    /// Make two vectors necessary to create lt-fm-index and locate pattern.
    ///     - The "joined_sequence" means the sequence of concatenated sequences of all record.
    ///     - The "accumulated_lengths" means the accumulated sequence lengths from 0 to the sum of the lengths of all sequences.
    ///     - For examples, if there are three records with "ATT", "CC", "GGGG", the "joined_sequence" is "ATTCCGGGG" and the "accumulated_lengths" is [0, 3, 5, 9].
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

/// Trait to provide label of record
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
        let amino_acid_with_noise_sequence_2 = b"AGCGTTTTATTACCTTTTGAATCCCAAAACATACATGCAGCATTCATTTTGCCACCAGTTTTTTTCATGCTTGATTCATATATAGCCTTTCTATCAGGAGATACTGTTTCTCCATGCTGCATACACAATTTTCGATAAGCATCATCATCCCTTTTTCCAGTAGCAAACTCTTTTCTTGCAAGTTCTTTTATTGCTTCGTCAAATTCTTCCTCTGACATCGCTGGTTTATCTCGTTTTGTCATGATAGTATCCCAGTTTGGTTTGGTAAAATTAATGTCCACAGGCTTAAATCTTAATGAGXXXMMN";

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

        assert_eq!(
            SequenceType::inferred_from_sequence(amino_acid_with_noise_sequence_2).unwrap(),
            SequenceType::aminoacid_with_noise(b'X'),
        );

        assert!(SequenceType::inferred_from_sequence(errored_sequence).is_err());
    }
}