use sigalign_core::reference::SequenceStorage;
use sigalign_impl::sequence_storage::in_memory::{
    InMemoryStorage,
    InMemoryBuffer,
};

mod in_memory_storage;

#[test]
fn provide_the_correct_sequence_with_id() {
    let mut in_memory_storage = InMemoryStorage::new();

}