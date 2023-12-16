use std::fs::File;
use std::io::Read;

use thiserror::Error;

use sigalign_core::reference::Reference as RawReference;
use sigalign_impl::{
    pattern_index::dynamic_lfi::{
        DynamicLfi, DynamicLfiOption, LfiBuildError,
    },
    sequence_storage::in_memory::{InMemoryStorage, InMemoryBuffer},
};

mod io;
pub use io::ReferenceLoadError;
mod debug;

/// A database for multiple target sequences.
pub struct Reference {
    raw_reference: RawReference<DynamicLfi, InMemoryStorage>,
    full_sorted_target_indices: Vec<u32>,
}

impl AsRef<RawReference<DynamicLfi, InMemoryStorage>> for Reference {
    fn as_ref(&self) -> &RawReference<DynamicLfi, InMemoryStorage> {
        &self.raw_reference
    }
}

impl Reference {
    /* Building Reference */
    /// ⚠️ This is lowest-level generator for `Reference`, assuming that users have already known about "sigalign-core" and "sigalign-impl" crates.
    pub fn from_raw(reference: RawReference<DynamicLfi, InMemoryStorage>) -> Self {
        let full_sorted_search_range = (0..reference.num_targets()).collect();
        Self { raw_reference: reference, full_sorted_target_indices: full_sorted_search_range }
    }
    /// Build `Reference` from a FASTA file (can be read from any `Read`).
    pub fn from_fasta<R: Read>(reader: R) -> Result<Self, ReferenceBuildError> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta(reader).map_err(|_| ReferenceBuildError::invalid_fasta_record())?;
        let dynamic_lfi_option = Self::get_option_for_dynamic_lfi(&sequence_storage);
        let raw_reference = RawReference::new(
            sequence_storage,
            dynamic_lfi_option,
        )?;
        Ok(Self::from_raw(raw_reference))
    }
    /// Build `Reference` from a FASTA file path.
    pub fn from_fasta_file<P>(path: P) -> Result<Self, ReferenceBuildError> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut sequence_storage = InMemoryStorage::new();
        let file = File::open(path)?;
        sequence_storage.add_fasta(file).map_err(|_| ReferenceBuildError::invalid_fasta_record())?;
        let dynamic_lfi_option = Self::get_option_for_dynamic_lfi(&sequence_storage);
        let raw_reference = RawReference::new(
            sequence_storage,
            dynamic_lfi_option,
        )?;
        Ok(Self::from_raw(raw_reference))
    }
    fn get_option_for_dynamic_lfi(sequence_storage: &InMemoryStorage) -> DynamicLfiOption {
        let total_length = sequence_storage.get_total_length();
        // Use 1/8 of total length as the maximum size of lookup table.
        // Maximum: 200 MiB
        let lookup_table_max_bytes_size = u64::min(
            200 * 1024 * 1024,
            (total_length / 8) as u64,
        );
        DynamicLfiOption {
            suffix_array_sampling_ratio: 1,
            lookup_table_max_bytes_size,
        }
    }

    /* Get Information */
    /// Get the sequence of the target. None if the target index is out of range.
    pub fn get_sequence(&self, target_index: u32) -> Option<Vec<u8>> {
        self.as_ref().get_sequence_storage().get_sequence_safely(target_index)
    }
    /// Get the label of the target. None if the target index is out of range.
    pub fn get_label(&self, target_index: u32) -> Option<String> {
        self.as_ref().get_sequence_storage().get_label_safely(target_index)
    }
    /// Get the number of targets.
    pub fn get_num_targets(&self) -> u32 {
        self.as_ref().num_targets()
    }
    /// Get the total length of all targets (in base pairs).
    pub fn get_total_length(&self) -> u32 {
        self.as_ref().get_sequence_storage().get_total_length()
    }

    /// Get sequence buffer for alignment.
    pub fn get_sequence_buffer(&self) -> InMemoryBuffer {
        self.as_ref().get_sequence_buffer()
    }
    /// Get the full sorted target indices
    pub fn get_full_sorted_target_indices(&self) -> &[u32] {
        &self.full_sorted_target_indices
    }
}

/// Error for building `Reference`.
#[derive(Error, Debug)]
pub enum ReferenceBuildError {
    #[error(transparent)]
    PatternIndexError(#[from] LfiBuildError),
    #[error("Invalid input: {0}")]
    InvalidSequence(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl ReferenceBuildError {
    // ID of FASTA record is invalid UTF8.
    fn invalid_fasta_record() -> Self {
        Self::InvalidSequence("ID of FASTA record is invalid UTF8".to_string())
    }
}
