use crate::test_data_path::get_lf_fa_path;
use super::{
    Reference,
    ReferenceBuilder,
    InMemoryStorage,
};
use std::io::Cursor;

#[test]
fn reference_serialization() {
    // First saved buffer
    let first_reference = get_test_reference();

    let mut buffer_1 = Vec::new();
    first_reference.save_to(&mut buffer_1).unwrap();
    
    let cursor = Cursor::new(buffer_1.clone());
    let second_reference: Reference<InMemoryStorage> = Reference::load_from(cursor).unwrap();

    // Second saved buffer
    let mut buffer_2 = Vec::new();
    second_reference.save_to(&mut buffer_2).unwrap();

    assert_eq!(buffer_1, buffer_2);
}

fn get_test_reference() -> Reference<InMemoryStorage> {
    let ref_file = get_lf_fa_path();
    let mut in_memory_storage = InMemoryStorage::new();
    in_memory_storage.add_fasta_file(ref_file).unwrap();

    ReferenceBuilder::new()
        .search_for_nucleotide_only()
        .build(in_memory_storage).unwrap()
}