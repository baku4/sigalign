use crate::reference::{
    Reference,
    pattern_index::pattern_indices::lfi::{Lfi32B2V64, LfiOption},
    sequence_storage::sequence_storages::InMemoryStorage,
};

pub struct ReferenceWrapper {
    inner: SelfDescReference,
}
enum SelfDescReference {
    Lfi32B2V64(Reference<Lfi32B2V64, InMemoryStorage>),
}

impl ReferenceWrapper {
    fn new() {
        let mut sequence_storage = InMemoryStorage::new();
        let option = LfiOption {
            suffix_array_sampling_ratio: 1,
            lookup_table_kmer_size: 2,
        };
        let reference = Reference::<Lfi32B2V64, InMemoryStorage>::new(sequence_storage, option);
    }
}