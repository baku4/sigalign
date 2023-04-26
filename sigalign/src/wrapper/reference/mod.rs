use crate::core::{
    ReferenceInterface,
    PatternLocation,
};
use crate::reference::{
    Reference, ReferenceBuildError,
    sequence_type::SequenceType,
    sequence_storage::{implementations::InMemoryStorage, SequenceStorage},
    features::Serialize,
};

mod pattern_index;
use pattern_index::LfiWrapper;

pub struct DefaultReference {
    inner: Reference<LfiWrapper, InMemoryStorage>,
}
impl DefaultReference {
    pub fn from_fasta_bytes(bytes: &[u8]) -> Result<Self, ReferenceBuildError> {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_bytes(bytes);
        Self::from_sequence_storage(sequence_storage)
    }
    pub fn from_fasta_file<P>(path: P) -> Result<Self, ReferenceBuildError> where
        P: AsRef<std::path::Path> + std::fmt::Debug,
    {
        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_file(path)?;
        Self::from_sequence_storage(sequence_storage)
    }
    fn from_sequence_storage(sequence_storage: InMemoryStorage) -> Result<Self, ReferenceBuildError> {
        let concatenated_sequence_with_boundaries = sequence_storage.get_concatenated_sequence_with_boundaries();
        let sequence_type = SequenceType::new(&concatenated_sequence_with_boundaries.concatenated_sequence);
        let num_targets = concatenated_sequence_with_boundaries.boundaries.len() - 1;
        let search_range: Vec<u32> = (0..num_targets as u32).collect();
        let pattern_index = LfiWrapper::new_with_default_option(concatenated_sequence_with_boundaries, &sequence_type)?;
        let inner = Reference::from_raw_unchecked(sequence_type, search_range, pattern_index, sequence_storage);
        
        Ok(Self { inner })
    }
}
impl ReferenceInterface for DefaultReference {
    type Buffer = <InMemoryStorage as SequenceStorage>::Buffer;

    #[inline]
    fn locate(&self, pattern: &[u8]) -> Vec<PatternLocation> {
        self.inner.locate(pattern)
    }
    #[inline]
    fn get_buffer(&self) -> Self::Buffer {
        self.inner.get_buffer()
    }
    #[inline]
    fn fill_buffer(&self, target_index: u32, buffer: &mut Self::Buffer) {
        self.inner.fill_buffer(target_index, buffer)
    }
    #[inline]
    fn is_valid(&self, query: &[u8]) -> bool {
        self.inner.is_valid(query)
    }
}
impl Serialize for DefaultReference {
    fn save_to<W>(&self, writer: W) -> Result<(), std::io::Error> where
        W: std::io::Write
    {
        self.inner.save_to(writer)
    }
    fn load_from<R>(reader: R) -> Result<Self, std::io::Error> where
        R: std::io::Read,
        Self: Sized,
    {
        let inner = Reference::load_from(reader)?;
        Ok(Self { inner })
    }
}