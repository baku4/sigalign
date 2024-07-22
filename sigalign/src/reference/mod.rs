use sigalign_core::reference::{
    Reference as RawReference,
    extensions::EstimateSize as _,
};
use sigalign_impl::{
    pattern_index::dynamic_lfi::DynamicLfi,
    sequence_storage::in_memory::{InMemoryStorage, InMemoryBuffer},
};
use crate::results::{
    QueryAlignment, TargetAlignment, LabeledQueryAlignment, LabeledTargetAlignment,
};

mod io;
pub use io::ReferenceLoadError;
mod debug;
mod builder;
pub use builder::{ReferenceBuilder, ReferenceBuildError};

pub type DefaultSequenceBuffer = InMemoryBuffer;
/// A database for multiple target sequences.
#[derive(Clone)]
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
    /// Get estimated size in bytes. (This is an estimate, not the exact size.)
    pub fn get_estimated_size_in_bytes(&self) -> usize {
        self.as_ref().serialized_size()
    }

    /* Access Resources */
    /// Get sequence buffer for alignment.
    pub fn get_sequence_buffer() -> InMemoryBuffer {
        InMemoryBuffer::new()
    }
    /// Get the full sorted target indices
    pub fn get_full_sorted_target_indices(&self) -> &[u32] {
        &self.full_sorted_target_indices
    }

    /* Manipulate Results */
    /// Label the query alignment.
    pub fn label_query_alignment(&self, query_alignment: QueryAlignment) -> LabeledQueryAlignment {
        let labeled_target_alignments = query_alignment.0.into_iter().map(|x| {
            self.label_target_alignment(x)
        }).collect();
        LabeledQueryAlignment(labeled_target_alignments)
    }
    /// Label the target alignment.
    #[inline]
    pub fn label_target_alignment(&self, target_alignment: TargetAlignment) -> LabeledTargetAlignment {
        let target_index = target_alignment.index;
        let label = self.get_label(target_index).unwrap_or_else(|| target_index.to_string());
        LabeledTargetAlignment {
            index: target_index,
            label,
            alignments: target_alignment.alignments,
        }
    }
}

impl Into<RawReference<DynamicLfi, InMemoryStorage>> for Reference {
    fn into(self) -> RawReference<DynamicLfi, InMemoryStorage> {
        self.raw_reference
    }
}
impl From<RawReference<DynamicLfi, InMemoryStorage>> for Reference {
    fn from(raw_reference: RawReference<DynamicLfi, InMemoryStorage>) -> Self {
        let full_sorted_target_indices = (0..raw_reference.num_targets()).collect();
        Self {
            raw_reference,
            full_sorted_target_indices,
        }
    }
}
