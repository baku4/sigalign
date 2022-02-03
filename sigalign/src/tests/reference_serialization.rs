use super::*;
use std::io::Cursor;


#[test]
fn test_reference_serialization() {
    let reference_to_save = get_reference();

    let mut buffer = Vec::new();

    reference_to_save.save_to(&mut buffer);

    let mut cursor = Cursor::new(buffer);

    let loaded_reference: Reference<InMemoryProvider> = Reference::load_from(cursor).unwrap();
}

fn get_reference() -> Reference<InMemoryProvider> {
    let reference_fasta_path = prepare_reference_1();

    let mut in_memory_provider = InMemoryProvider::new();
    in_memory_provider.add_fasta_file(&reference_fasta_path).unwrap();

    ReferenceBuilder::new()
        .search_for_nucleotide_only()
        .build(in_memory_provider).unwrap()
}