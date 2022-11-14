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
    Serializable,
    RcStorage,
    Divisible,

    InMemoryStorage,
    InMemoryRcStorage,
    IndexedFastaStorage,
    IndexedFastaRcStorage,
};

#[test]
fn sequence_storages_provide_same_sequence() {
    for fa in [get_lf_fa_path(), get_crlf_fa_path(), get_two_line_fa_path()] {
        // Make SequenceStorage
        // (1) In memory storage
        let mut in_memory_storage = InMemoryStorage::new();
        in_memory_storage.add_fasta_file(&fa).unwrap();
        let mut in_memory_rc_storage = InMemoryRcStorage::new();
        in_memory_rc_storage.add_fasta_file(&fa).unwrap();
        // (2) Indexed fasta storage
        let indexed_fasta_storage = IndexedFastaStorage::new(&fa).unwrap();
        let indexed_fasta_rc_storage = IndexedFastaRcStorage::new(&fa).unwrap();

        assert_both_provide_same_sequence(&in_memory_storage, &indexed_fasta_storage);
        assert_both_provide_same_sequence(&in_memory_rc_storage, &indexed_fasta_rc_storage);
        
        // assert_both_provide_same_label(&in_memory_storage, &indexed_fasta_storage);
        // assert_both_provide_same_label(&in_memory_rc_storage, &indexed_fasta_rc_storage);

        assert_both_provide_same_rc(&in_memory_rc_storage, &indexed_fasta_rc_storage);
    }
}

pub fn assert_both_provide_same_sequence<S1, S2>(
    sequence_storage_1: &S1,
    sequence_storage_2: &S2,
) where
    S1: SequenceStorage,
    S2: SequenceStorage,
{
    let read_count_1 = sequence_storage_1.total_record_count();
    let read_count_2 = sequence_storage_2.total_record_count();

    assert_eq!(read_count_1, read_count_2);

    let mut sequence_buffer_1 = sequence_storage_1.get_buffer();
    let mut sequence_buffer_2 = sequence_storage_2.get_buffer();
    for record_index in 0..read_count_1 {
        sequence_storage_1.fill_sequence_buffer(record_index, &mut sequence_buffer_1);
        sequence_storage_2.fill_sequence_buffer(record_index, &mut sequence_buffer_2);

        let sequence_1 = String::from_utf8(sequence_buffer_1.request_sequence().to_vec()).unwrap();
        let sequence_2 = String::from_utf8(sequence_buffer_2.request_sequence().to_vec()).unwrap();

        assert_eq!(sequence_1, sequence_2);
    }
}

pub fn assert_both_provide_same_label<SL1, SL2>(
    sequence_storage_1: &SL1,
    sequence_storage_2: &SL2,
) where
    SL1: SequenceStorage + LabelStorage,
    SL2: SequenceStorage + LabelStorage,
{
    let read_count_1 = sequence_storage_1.total_record_count();
    let read_count_2 = sequence_storage_2.total_record_count();
    
    assert_eq!(read_count_1, read_count_2);

    for record_index in 0..read_count_1 {
        let label_1 = sequence_storage_1.label_of_record(record_index);
        let label_2 = sequence_storage_2.label_of_record(record_index);

        assert_eq!(label_1, label_2);
    }
}

fn assert_both_provide_same_rc<SR1, SR2>(
    sequence_storage_1: &SR1,
    sequence_storage_2: &SR2,
) where
    SR1: SequenceStorage + RcStorage,
    SR2: SequenceStorage + RcStorage,
{
    let read_count_1 = sequence_storage_1.total_record_count();
    let read_count_2 = sequence_storage_2.total_record_count();
    
    assert_eq!(read_count_1, read_count_2);

    for record_index in 0..read_count_1 {
        let is_rc_1 = sequence_storage_1.is_reverse_complement(record_index);
        let is_rc_2 = sequence_storage_2.is_reverse_complement(record_index);

        assert_eq!(is_rc_1, is_rc_2);
    }
}
