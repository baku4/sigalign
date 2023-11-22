use std::fs::File;
use std::io::Read;

use thiserror::Error;

use sigalign_core::reference::Reference as RawReference;
use sigalign_impl::{
    pattern_index::dynamic_lfi::{
        DynamicLfi, DynamicLfiOption, LfiBuildError,
    },
    sequence_storage::in_memory::InMemoryStorage,
};

mod io;
pub use io::ReferenceLoadError;
// mod debug

pub struct Reference {
    raw_reference: RawReference<DynamicLfi, InMemoryStorage>,
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
        Self { raw_reference: reference }
    }
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
        // Use half of total length as the maximum size of lookup table.
        // Maximum: 200 Mb (200_000_000)
        let lookup_table_max_bytes_size = u64::max(
            200_000_000,
            (total_length / 2) as u64,
        );
        DynamicLfiOption {
            suffix_array_sampling_ratio: 1,
            lookup_table_max_bytes_size,
        }
    }

    /* Get Information */
    pub fn get_sequence(&self, target_index: u32) -> Option<Vec<u8>> {
        self.as_ref().get_sequence_storage().get_sequence_safely(target_index)
    }
    pub fn get_label(&self, target_index: u32) -> Option<String> {
        self.as_ref().get_sequence_storage().get_label_safely(target_index)
    }
    pub fn get_num_targets(&self) -> u32 {
        self.as_ref().num_targets()
    }
    pub fn get_total_length(&self) -> u32 {
        self.as_ref().get_sequence_storage().get_total_length()
    }
}

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