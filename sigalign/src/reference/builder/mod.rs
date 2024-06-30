use std::{io::Read, fs::File};

use thiserror::Error;

use sigalign_core::reference::Reference as RawReference;
use sigalign_impl::{
    pattern_index::dynamic_lfi::{
        DynamicLfiOption, LfiBuildError,
    },
    sequence_storage::in_memory::InMemoryStorage,
};
use super::Reference;

/// Builder for `Reference`.
/// 
/// - Default configuration:
///   - Uppercase: false
///      - Reference treats uppercase and lowercase letters as different bases.
///   - Ignore bases: None
///      - Reference treats all characters as bases.
pub struct ReferenceBuilder {
    uppercase: bool,
    to_ignore_bases: Vec<u8>,
    sequence_storage: InMemoryStorage,
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
    #[error("Sequence is empty")]
    EmptySequence,
}

impl ReferenceBuilder {
    /// Make a new `ReferenceBuilder`.
    pub fn new() -> Self {
        Self {
            uppercase: true,
            to_ignore_bases: Vec::new(),
            sequence_storage: InMemoryStorage::new(),
        }
    }
    /* Configuration */
    /// Set all letters to uppercase when building.
    pub fn set_uppercase(mut self, uppercase: bool) -> Self {
        self.uppercase = uppercase;
        self
    }
    /// Set the base that never match to any other bases.
    pub fn ignore_base(mut self, base: u8) -> Self {
        self.to_ignore_bases.push(base);
        self
    }
    /// Set the bases that never match to any other bases (multiple).
    pub fn ignore_bases(mut self, bases: &[u8]) -> Self {
        self.to_ignore_bases.extend_from_slice(bases);
        self
    }
    /// Reset the bases to ignore.
    pub fn reset_ignore_bases(mut self) -> Self {
        self.to_ignore_bases.clear();
        self
    }
    /* Add Sequences */
    pub fn add_target(mut self, label: &str, sequence: &[u8]) -> Self {
        self.sequence_storage.add_target(label, sequence);
        self
    }
    pub fn add_fasta<R: Read>(mut self, reader: R) -> Result<Self, ReferenceBuildError> {
        self.sequence_storage.add_fasta(reader).map_err(|_| ReferenceBuildError::invalid_fasta_record())?;
        Ok(self)
    }
    pub fn add_fasta_file<P>(mut self, path: P) -> Result<Self, ReferenceBuildError> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let file = File::open(path)?;
        self.sequence_storage.add_fasta(file).map_err(|_| ReferenceBuildError::invalid_fasta_record())?;
        Ok(self)
    }

    /// Finish building `Reference`.
    pub fn build(mut self) -> Result<Reference, ReferenceBuildError> {
        // Sequence Storage
        if self.uppercase {
            self.sequence_storage.set_sequences_to_uppercase()
        }
        if !self.to_ignore_bases.is_empty() {
            self.sequence_storage.change_bases_to(&self.to_ignore_bases, b'?');
        }

        // Pattern index option
        let dynamic_lfi_option = Self::get_option_for_dynamic_lfi(&self.sequence_storage);
        let raw_reference = RawReference::new(
            self.sequence_storage,
            dynamic_lfi_option,
        )?;
        Ok(Reference::from(raw_reference))
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
            use_safe_guard: true,
        }
    }
}

impl ReferenceBuildError {
    // ID of FASTA record is invalid UTF8.
    fn invalid_fasta_record() -> Self {
        Self::InvalidSequence("ID of FASTA record is invalid UTF8".to_string())
    }
}
