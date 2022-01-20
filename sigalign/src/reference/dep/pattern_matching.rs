use super::{Sequence};

use lt_fm_index::{FmIndex, LtFmIndexAll, LtFmIndexConfig};
use lt_fm_index::use_case::*;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct LtFmIndex(LtFmIndexAll);

impl LtFmIndex {
    pub fn new(
        is_nucleotide: bool,
        with_noise: bool,
        use_bwt_size_of_128: bool,
        sa_sampling_ratio: u64,
        kmer_size_for_lookup_table: Option<usize>,
        text: Vec<u8>,
    ) -> Self {
        let mut lt_fm_index_config = if is_nucleotide {
            LtFmIndexConfig::for_nucleotide()
        } else {
            LtFmIndexConfig::for_aminoacid()
        };

        if with_noise {
            lt_fm_index_config = lt_fm_index_config.with_noise();
        }

        if use_bwt_size_of_128 {
            lt_fm_index_config = lt_fm_index_config.change_bwt_interval_to_128();
        }

        if let Some(kmer) = kmer_size_for_lookup_table {
            lt_fm_index_config = lt_fm_index_config.change_kmer_size(kmer).unwrap();
        }

        Self(
            lt_fm_index_config.change_sampling_ratio(sa_sampling_ratio).unwrap().generate(text).unwrap()
        )
    }
    pub fn locate(&self, pattern: Sequence) -> Vec<u64> {
        self.0.locate(pattern)
    }
    pub fn sa_sampling_ratio(&self) -> u64 {
        match &self.0 {
            LtFmIndexAll::NO64(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::NO128(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::NN64(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::NN128(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::AO64(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::AO128(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::AN64(inner_index) => inner_index.suffix_array_sampling_ratio(),
            LtFmIndexAll::AN128(inner_index) => inner_index.suffix_array_sampling_ratio(),
        }
    }
    pub fn kmer_size(&self) -> usize {
        match &self.0 {
            LtFmIndexAll::NO64(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::NO128(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::NN64(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::NN128(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::AO64(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::AO128(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::AN64(inner_index) => inner_index.lookup_table_kmer_size(),
            LtFmIndexAll::AN128(inner_index) => inner_index.lookup_table_kmer_size(),
        }
    }
    pub fn bwt_size(&self) -> usize {
        match &self.0 {
            LtFmIndexAll::NO64(_)
            | LtFmIndexAll::NN64(_)
            | LtFmIndexAll::AO64(_)
            | LtFmIndexAll::AN64(_) => 64,
            LtFmIndexAll::NO128(_)
            | LtFmIndexAll::NN128(_)
            | LtFmIndexAll::AO128(_)
            | LtFmIndexAll::AN128(_) => 128,
        }
    }
}
