use super::{
    PatternFinder,
};

use lt_fm_index::BwtCompressionSize;

impl PatternFinder {
    pub fn get_suffix_array_sampling_ratio(&self) -> u64 {
        self.lt_fm_index.suffix_array_sampling_ratio()
    }
    pub fn get_size_of_bwt_block(&self) -> usize {
        match self.lt_fm_index.bwt_compression_size() {
            BwtCompressionSize::_64 => 64,
            BwtCompressionSize::_128 => 128,
        }
    }
    pub fn get_lookup_table_kmer_size(&self) -> usize {
        self.lt_fm_index.lookup_table_kmer_size()
    }
}
