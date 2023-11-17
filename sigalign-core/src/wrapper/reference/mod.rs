use crate::reference::{
    Reference, ReferenceBuildError,
    sequence_storage::{in_memory::InMemoryStorage},
    pattern_index::lfi::{DynamicLfi, DynamicLfiOption},
};

pub type DefaultReference = Reference<DynamicLfi, InMemoryStorage>;

impl DefaultReference {
    pub fn from_fasta_bytes(bytes: &[u8]) -> Result<Self, ReferenceBuildError> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_bytes(bytes);
        Reference::new(
            sequence_storage,
            default_option_for_dynamic_lfi(),
        )
    }
    pub fn from_fasta_file<P>(path: P) -> Result<Self, ReferenceBuildError> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_file(path)?;
        Reference::new(
            sequence_storage,
            default_option_for_dynamic_lfi(),
        )
    }
    pub fn get_sequence(&self, target_index: u32) -> Option<Vec<u8>> {
        self.sequence_storage.get_sequence_safely(target_index)
    }
    pub fn get_label(&self, target_index: u32) -> Option<String> {
        self.sequence_storage.get_label_safely(target_index)
    }
}

fn default_option_for_dynamic_lfi() -> DynamicLfiOption {
    DynamicLfiOption {
        suffix_array_sampling_ratio: 1,
        max_lookup_table_byte_size: 3_000_000 // 3MB
    }
}
