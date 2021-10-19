use super::*;

use lt_fm_index::{LtFmIndexConfig, LtFmIndexAll};
use lt_fm_index::use_case::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct FmIndex {
    lt_fm_index: LtFmIndexAll,
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
