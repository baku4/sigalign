pub mod sequence_provider;

use crate::deprecated::alignment::Aligner;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use lt_fm_index::{LtFmIndex, LtFmIndexBuilder};

/// Records of Sequences
pub trait SequenceProvider<'a> {
    fn len(&'a self) -> usize;
    fn sequence(&'a self, index: usize) -> &'a [u8];
    fn label(&'a self, index: usize) -> &'a str;
    fn concated_sequence(&'a self) -> Vec<u8>;
    fn accumulated_length(&'a self) -> AccumulatedLength;
}

/// Accumulated length for locating k-sized pattern
/// (start, end)
pub type AccumulatedLength = Vec<(u64, u64)>;

/// Search Range  
/// must be sorted
pub type SearchRange = Vec<usize>;

/// Direction for represent reverse complement
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Direction {
    Forward,
    Reverse,
}

/*****************************
********* DATA BASE **********
*****************************/

/// Config
pub struct DatabaseConfig {
    reverse_complement: bool,
    in_memory_index: bool,
    // Lt-fm-index
    klt_kmer: usize,
    sa_sampling_ratio: u64,
    only_nucleotide: bool,
}
impl DatabaseConfig {
    // Settings
    pub fn new() -> Self {
        Self {
            reverse_complement: true,
            in_memory_index: true,
            klt_kmer: 10,
            sa_sampling_ratio: 2,
            only_nucleotide: true,
        }
    }
    // Create DB
    pub fn create_db<'a, P: SequenceProvider<'a>>(&self, sequence_provider: &'a P) -> Database<'a> {
        Database::new(self, sequence_provider)
    }
}

/// Database
pub struct Database<'a> {
    sequence_provider: &'a dyn SequenceProvider<'a>,
    // Index
    fm_index: LtFmIndex,
    accumulated_length: AccumulatedLength,
    // DB options
    in_memory_index: bool,
    reverse_complement: bool,
    only_nucleotide: bool,
    klt_kmer: usize,
    sa_sampling_ratio: u64,
}

impl<'a> Database<'a> {
    pub fn new<P: SequenceProvider<'a>>(database_config: &DatabaseConfig, sequence_provider: &'a P) -> Self {
        let concated_seq = sequence_provider.concated_sequence();
        let accumualated_length = sequence_provider.accumulated_length();
        let mut fm_index_config = LtFmIndexBuilder::new()
            .set_suffix_array_sampling_ratio(database_config.sa_sampling_ratio).unwrap()
            .set_lookup_table_kmer_size(database_config.klt_kmer).unwrap();
        if !database_config.only_nucleotide {
            fm_index_config = fm_index_config.use_nucleotide_only(); //TODO: change configs setting method
        } else {
            fm_index_config = fm_index_config.use_nucleotide_with_noise();
        }
        let fm_index = fm_index_config.build(concated_seq);
        Self {
            sequence_provider: sequence_provider,
            fm_index: fm_index,
            accumulated_length: accumualated_length,
            in_memory_index: database_config.in_memory_index,
            reverse_complement: database_config.reverse_complement,
            only_nucleotide: database_config.only_nucleotide,
            klt_kmer: database_config.klt_kmer,
            sa_sampling_ratio: database_config.sa_sampling_ratio,
        }
    }
    pub fn load<P: SequenceProvider<'a>>(file_path: &str, sequence_provider: &'a P) {

    }
    pub fn state(&self) {

    }
    pub fn search(
        &self,
        query: &[u8],
        aligner: &Aligner,
        search_range: &SearchRange,
        get_minimum_penalty: bool,
    ) {
        let res_for_db = aligner.alignment_with_query(
            self,
            search_range,
            query,
            get_minimum_penalty,
        );
        ()
    }
    pub fn locate(&self, pattern: &[u8]) -> Vec<u64> {
        self.fm_index.locate(pattern) //TODO: locate
    }
    pub fn get_range(&self) -> Vec<usize> {
        (0..self.accumulated_length.len()).collect()
    }
    pub fn get_ref_len(&self, ref_index: usize) -> usize {
        let (start, end) = self.accumulated_length[ref_index];
        (end - start) as usize
    }
    pub fn get_sequence(&self, ref_index: usize) -> &[u8] {
        self.sequence_provider.sequence(ref_index)
    }
    pub fn find_ref_positions(
        &self,
        search_range: &SearchRange,
        sorted_positions: Vec<u64>,
        kmer: u64
    ) -> HashMap<usize, Vec<usize>> { // ref (index, pos)
        let mut ref_positions_by_index: HashMap<usize, Vec<usize>> = HashMap::with_capacity(sorted_positions.len()); // index and positions

        let mut size;
        let mut left;
        let mut right;
        let mut mid = 0;
        let mut index;

        for position in sorted_positions {
            // reset
            right = search_range.len();
            left = mid;
            size = right - left;
    
            while left < right {
                mid = left + size / 2;
                index = search_range[mid];
                
                let (start, end) = self.accumulated_length[index];
                if position >= end {
                    left = mid + 1;
                } else if start > position {
                    right = mid;
                } else {
                    if (position + kmer) < end {
                        let ref_pos = (position - start) as usize;
                        match ref_positions_by_index.get_mut(&index) {
                            Some(v) => {
                                v.push(ref_pos);
                            },
                            None => {
                                ref_positions_by_index.insert(index, vec![ref_pos]);
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
    
        ref_positions_by_index
    }
}

// TODO: to del
// #[derive(Debug, Serialize, Deserialize, PartialEq)]
// pub struct SerializedDatabase {
//     // Index
//     fm_index: LtFmIndex,
//     accumulated_length: AccumulatedLength,
//     // DB options
//     in_memory_index: bool,
//     reverse_complement: bool,
//     only_nucleotide: bool,
//     klt_kmer: usize,
//     sa_sampling_ratio: u64,
// }


#[cfg(test)]
mod tests {
    use super::*;
    
    fn test_create_db() {
        let reverse_complement = true;

        let ref_fasta = "./src/deprecated/tests/fasta/ERR209055.fa";

        let (seq_provider, _) = sequence_provider::OnMemoryProvider::from_fasta(
            reverse_complement,
            ref_fasta
        );

        let database_config = DatabaseConfig::new();

        let database = database_config.create_db(&seq_provider);
    }
}