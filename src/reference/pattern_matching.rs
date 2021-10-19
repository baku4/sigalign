use super::*;

use lt_fm_index::{FmIndex, LtFmIndexAll, LtFmIndexConfig};
use lt_fm_index::use_case::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct LtFmIndex {
    lt_fm_index: LtFmIndexAll,
}
impl LtFmIndex {
    pub fn locate(&self, pattern: &[u8]) -> Vec<u64> {
        self.lt_fm_index.locate(pattern)
    }
}

struct FmIndexOption {
    sampling_ratio: usize,
    kmer_size: usize,
    bwt_size: BwtSize,
}

enum BwtSize {
    _64,
    _128,
}
