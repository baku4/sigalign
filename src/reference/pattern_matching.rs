use super::*;

use lt_fm_index::{FmIndex, LtFmIndexAll, LtFmIndexConfig};
use lt_fm_index::use_case::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LtFmIndex {
    lt_fm_index: LtFmIndexAll,
}
impl LtFmIndex {
    pub fn new(
        is_nucleotide: bool,
        use_bit_size_of_64: bool,
        sa_sampling_ratio: u64,
        kmer_size_for_lookup_table: usize,
        text: Vec<u8>,
    ) -> Self {
        if is_nucleotide {
            if use_bit_size_of_64 {
                LtFmIndexNN64::new(text, sa_sampling_ratio, kmer_size_for_lookup_table)
            } else {
                //
            }
        } else {
            if use_bit_size_of_64 {
                //
            } else {
                //
            }
        }
    }
    pub fn locate(&self, pattern: &[u8]) -> Vec<u64> {
        self.lt_fm_index.locate(pattern)
    }
}
