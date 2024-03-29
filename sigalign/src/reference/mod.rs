use sigalign_core::reference::Reference as RawReference;
use sigalign_impl::{
    pattern_index::dynamic_lfi::DynamicLfi,
    sequence_storage::in_memory::{InMemoryStorage, InMemoryBuffer},
};

mod io;
pub use io::ReferenceLoadError;
mod debug;
mod builder;
pub use builder::{ReferenceBuilder, ReferenceBuildError};

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
    pub fn get_sequence_buffer() -> InMemoryBuffer {
        InMemoryBuffer::new()
    }
    /// Get the full sorted target indices
    pub fn get_full_sorted_target_indices(&self) -> &[u32] {
        &self.full_sorted_target_indices
    }
}
