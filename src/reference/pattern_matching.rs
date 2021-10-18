use super::*;

use lt_fm_index::{LtFmIndexConfig, LtFmIndexAll, IO};
use lt_fm_index::use_case::*;

struct FmIndex {
    lt_fm_index: LtFmIndexConfig,
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
