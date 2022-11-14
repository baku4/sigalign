use crate::{
    Result, error_msg,
    init_logger,
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
use crate::random_text::{
    rand_text_of_nn,
};
use sigalign::util::reverse_complement_of_nucleotide_sequence;
use log::info;

#[test]
fn rc_from_in_memory_storage_is_collect() {
    init_logger();

    let record_count = 10;
    let sequence_list: Vec<Vec<u8>> = (0..record_count).map(|_| {
        rand_text_of_nn()
    }).collect();
    let label_list: Vec<String> = (0..record_count).map(|idx| {
        format!("idx_{}", idx)
    }).collect();

    let mut ims = InMemoryStorage::new();
    for idx in 0..record_count {
        ims.add_record(&sequence_list[idx], &label_list[idx]);
    }

    let rc_ims = ims.to_reverse_complement();

    let mut org_buffer = ims.get_buffer();
    let mut rc_buffer = rc_ims.get_buffer();

    info!("Total record count: {}, NN type", record_count);
    for idx in 0..record_count {
        info!("idx: {}", idx);
        // Sequence
        ims.fill_sequence_buffer(idx, &mut org_buffer);
        rc_ims.fill_sequence_buffer(idx, &mut rc_buffer);
        let org_seq = org_buffer.request_sequence();
        let rc_from_org_seq = reverse_complement_of_nucleotide_sequence(org_seq);
        let rc_seq = rc_buffer.request_sequence();
        assert_eq!(&rc_from_org_seq, rc_seq);

        // Label
        let org_label = ims.label_of_record(idx);
        let rc_label = rc_ims.label_of_record(idx);
        assert_eq!(org_label, rc_label);
    }
}

#[test]
fn merged_in_memory_storage_is_collect() {
    init_logger();

    let first_record_count = 12;
    let first_sequence_list: Vec<Vec<u8>> = (0..first_record_count).map(|_| {
        rand_text_of_nn()
    }).collect();
    let first_label_list: Vec<String> = (0..first_record_count).map(|idx| {
        format!("first_{}", idx)
    }).collect();
    let mut first_ims = InMemoryStorage::new();
    for idx in 0..first_record_count {
        first_ims.add_record(&first_sequence_list[idx], &first_label_list[idx]);
    }

    let second_record_count = 10;
    let second_sequence_list: Vec<Vec<u8>> = (0..second_record_count).map(|_| {
        rand_text_of_nn()
    }).collect();
    let second_label_list: Vec<String> = (0..second_record_count).map(|idx| {
        format!("second_{}", idx)
    }).collect();
    let mut second_ims = InMemoryStorage::new();
    for idx in 0..second_record_count {
        second_ims.add_record(&second_sequence_list[idx], &second_label_list[idx]);
    }

    first_ims.merge(second_ims);

    let mut buffer = first_ims.get_buffer();

    info!("Total record count: first-{}, second-{}; NN type", first_record_count, second_record_count);
    for idx in 0..first_record_count {
        info!("first idx {}", idx);
        first_ims.fill_sequence_buffer(idx, &mut buffer);
        let seq = buffer.request_sequence();
        let label = first_ims.label_of_record(idx);

        assert_eq!(seq, &first_sequence_list[idx]);
        assert_eq!(&label, &first_label_list[idx]);
    }
    for idx in 0..second_record_count {
        info!("second idx {}", idx);
        let idx_in_first_ims = idx + first_record_count;
        
        first_ims.fill_sequence_buffer(idx_in_first_ims, &mut buffer);
        let seq = buffer.request_sequence();
        let label = first_ims.label_of_record(idx_in_first_ims);

        assert_eq!(seq, &second_sequence_list[idx]);
        assert_eq!(&label, &second_label_list[idx]);
    }
}