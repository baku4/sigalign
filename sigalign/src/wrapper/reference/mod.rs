use crate::reference::{
    Reference, ReferenceBuildError,
    sequence_storage::{in_memory::InMemoryStorage},
};

mod pattern_index;
pub use pattern_index::{LfiWrapper, LfiWrapperOption};

pub type DefaultReference = Reference<LfiWrapper, InMemoryStorage>;

impl DefaultReference {
    pub fn from_fasta_bytes(bytes: &[u8]) -> Result<Self, ReferenceBuildError> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_bytes(bytes);
        Reference::new(
            sequence_storage,
            LfiWrapperOption,
        )
    }
    pub fn from_fasta_file<P>(path: P) -> Result<Self, ReferenceBuildError> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_file(path)?;
        Reference::new(
            sequence_storage,
            LfiWrapperOption,
        )
    }
}
