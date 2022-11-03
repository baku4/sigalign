#[allow(unused_imports)]
use super::{
    Result, error_msg,
    get_lf_fa_path,
    get_crlf_fa_path,
    get_two_line_fa_path,
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
use std::io::Cursor;

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

fn assert_both_provide_same_sequence<S1, S2>(
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

fn assert_both_provide_same_label<SL1, SL2>(
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

fn assert_storage_serialization<S>(
    sequence_storage_to_save: &S,
) where
    S: SequenceStorage + Serializable,
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

// Test divisible
// TODO: To modify (C&P from original crate)
fn check_splitted_storage_with_fasta_file(fasta_file: &str) {
    // Original
    let mut in_memory_storage = InMemoryRcStorage::new();
    in_memory_storage.add_fasta_file(fasta_file);
    let mut org_label_list: Vec<String> = Vec::with_capacity(in_memory_storage.total_record_count());
    let mut org_seq_list: Vec<Vec<u8>> = Vec::with_capacity(in_memory_storage.total_record_count());
    for idx in 0..in_memory_storage.total_record_count() {
        // Label
        let label = in_memory_storage.label_of_record(idx);
        org_label_list.push(label);
        // Seq
        let mut buffer = in_memory_storage.get_buffer();
        in_memory_storage.fill_sequence_buffer(idx, &mut buffer);
        let seq = buffer.request_sequence().to_vec();
        org_seq_list.push(seq);
    }

    // Splitted
    let mut splitted_label_list: Vec<String> = Vec::with_capacity(in_memory_storage.total_record_count());
    let mut splitted_seq_list: Vec<Vec<u8>> = Vec::with_capacity(in_memory_storage.total_record_count());

    let splitted = in_memory_storage.split_by_max_length(10000).unwrap();
    println!("splitted_len: {}", splitted.len());

    for (idx, in_memory_storage) in splitted.into_iter().enumerate() {
        for ridx in 0..in_memory_storage.total_record_count() {
            let label = in_memory_storage.label_of_record(ridx);
            splitted_label_list.push(label);

            let mut buffer = in_memory_storage.get_buffer();
            in_memory_storage.fill_sequence_buffer(ridx, &mut buffer);
            // let seq = String::from_utf8(buffer.request_sequence().to_vec()).unwrap();
            let seq = buffer.request_sequence().to_vec();
            splitted_seq_list.push(seq);
        }
    }

    // Compare
    for idx in 0..org_label_list.len() {
        assert_eq!(org_label_list[idx], splitted_label_list[idx]);
        assert_eq!(org_seq_list[idx], splitted_seq_list[idx])
    }
}