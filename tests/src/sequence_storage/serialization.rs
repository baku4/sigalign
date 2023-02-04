use crate::{
    Result, error_msg,
    test_data::{
        get_lf_fa_path,
        get_crlf_fa_path,
        get_two_line_fa_path,
    },
};
use super::{
    SequenceBuffer,
    SequenceStorage,
    LabelStorage,
    Serialize,
    RcStorage,
    Divide,

    InMemoryStorage,
    InMemoryRcStorage,
    IndexedFastaStorage,
    IndexedFastaRcStorage,
};
use super::{
    assert_both_provide_same_sequence,
    assert_both_provide_same_label,
};
use std::io::Cursor;

#[test]
fn sequence_storages_serialization() {
    for fa in [get_lf_fa_path(), get_crlf_fa_path(), get_two_line_fa_path()] {
        // (1) In memory storage
        {
            let mut in_memory_storage = InMemoryStorage::new();
            in_memory_storage.add_fasta_file(&fa).unwrap();
            assert_storage_serialization(&in_memory_storage);
        }
        {
            let mut in_memory_rc_storage = InMemoryRcStorage::new();
            in_memory_rc_storage.add_fasta_file(&fa).unwrap();
            assert_storage_serialization(&in_memory_rc_storage);
        }
        // (2) Indexed fasta storage
        {
            let indexed_fasta_storage = IndexedFastaStorage::new(&fa).unwrap();
            assert_storage_serialization(&indexed_fasta_storage);
        }
        {
            let indexed_fasta_rc_storage = IndexedFastaRcStorage::new(&fa).unwrap();
            assert_storage_serialization(&indexed_fasta_rc_storage);
        }
    }
}

fn assert_storage_serialization<S>(
    sequence_storage_to_save: &S,
) where
    S: SequenceStorage + Serialize,
{
    // Save
    let mut buffer = Vec::new();
    sequence_storage_to_save.save_to(&mut buffer).unwrap();

    // Load
    let buffer_cursor = Cursor::new(buffer);
    let loaded_sequence_storage = S::load_from(buffer_cursor).unwrap();
    
    // trait 'PatialEq' impl is not guaranteed
    assert_both_provide_same_sequence(sequence_storage_to_save, &loaded_sequence_storage)
}
