use crate::test_data_path::get_lf_fa_path;
use sigalign::{
    wrapper::DefaultReference,
    reference::features::Serialize,
};
use std::io::Cursor;

#[test]
fn default_reference_serialization() {
    // First saved buffer
    let default_reference = get_default_reference();

    let mut buffer_1 = Vec::new();
    default_reference.save_to(&mut buffer_1).unwrap();
    
    let cursor = Cursor::new(buffer_1.clone());
    let loaded = DefaultReference::load_from(cursor).unwrap();

    // Second saved buffer
    let mut buffer_2 = Vec::new();
    loaded.save_to(&mut buffer_2).unwrap();

    assert_eq!(buffer_1, buffer_2);
}

fn get_default_reference() -> DefaultReference {
    let ref_file = get_lf_fa_path();
    let reference = DefaultReference::from_fasta_file(ref_file).unwrap();
    reference
}