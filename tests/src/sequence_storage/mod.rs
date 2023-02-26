#[allow(unused_imports)]
use crate::{
    Result, error_msg,
    test_data_path::{
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

mod provide_same_information;
use provide_same_information::{
    assert_both_provide_same_sequence,
    assert_both_provide_same_label
};
mod serialization;

// For each storage
mod in_memory;

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