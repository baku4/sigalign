mod sequence_provider;

use crate::alignment::Aligner;

use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use lt_fm_index::FmIndex;

/// Records of Sequences
pub trait SequenceProvider<'a> {
    fn len(&'a self) -> usize;
    fn sequence(&'a self, index: usize) -> &'a [u8];
    fn label(&'a self, index: usize) -> &'a str;
    fn concated_sequence(&'a self) -> Vec<u8>;
    fn accumulated_length(&'a self) -> AccumulatedLength;
}

/// Config for [Database]
pub struct DatabaseConfig {
    reverse_complement: bool,
    in_memory_sr: bool,
    in_memory_index: bool,
    // Lt-fm-index
    klt_kmer: usize,
    sa_sampling_ratio: u64,
    only_nucleotide: bool,
}
impl DatabaseConfig {
    pub fn new() -> Self {
        Self {
            reverse_complement: true,
            in_memory_sr: true,
            in_memory_index: true,
            klt_kmer: 13,
            sa_sampling_ratio: 2,
            only_nucleotide: true,
        }
    }
    pub fn create_db(&self) { //  -> Database
        // Database::new(self)
    }
}

/// Database
pub struct Database<'a> {
    sequence_provider: &'a dyn SequenceProvider<'a>,
    // Index
    fm_index: FmIndex,
    accumulated_length: AccumulatedLength,
    // DB options
    in_memory_index: bool,
    reverse_complement: bool,
    only_nucleotide: bool,
    klt_kmer: usize,
    sa_sampling_ratio: u64,
}

impl<'a> Database<'a> {
    pub fn new(database_config: &DatabaseConfig) {
        
    }
    pub fn load() {

    }
    pub fn state(&self) {

    }
    pub fn search(&self, query: &[u8], aligner: &Aligner, search_range: &SearchRange) {

    }
    pub fn locate(&self, pattern: &[u8]) -> Vec<u64> {
        self.fm_index.locate_w_klt(pattern) //TODO: locate
    }
    pub fn get_ref_len(&self, ref_index: usize) -> usize {
        let (start, end) = self.accumulated_length[ref_index];
        (end - start) as usize
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
            size = search_range.len();
            left = mid;
            right = size;
    
            while left < right {
                mid = left + size / 2;
                index = search_range[mid];
    
                let (start, end) = self.accumulated_length[index];
                if start > position {
                    left = mid + 1;
                } else if position >= end {
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

/// Accumulated length for locating k-sized pattern
/// (start, end)
pub type AccumulatedLength = Vec<(u64, u64)>;

/// Search Range  
/// ! must be sorted
pub type SearchRange = Vec<usize>;

/// Location of the database
/// (index of sequence, start position of pattern)
pub struct Location {
    pub index: usize,
    pub position: usize,
}

/// Result of search
struct SearchResult {
    label: String,
    sequence: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SerializedDatabase {
    // Index
    fm_index: FmIndex,
    accumulated_length: AccumulatedLength,
    // DB options
    in_memory_index: bool,
    reverse_complement: bool,
    only_nucleotide: bool,
    klt_kmer: usize,
    sa_sampling_ratio: u64,
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;
    
    #[test]
    fn test_find_ref_positions() {

    }
}