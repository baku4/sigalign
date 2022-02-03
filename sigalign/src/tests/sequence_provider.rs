use super::*;
use std::io::Cursor;

#[test]
fn test_sequence_providers_provide_same_sequence() {
    let fasta_file_path = NUCLEOTIDE_ONLY_FA_PATH_1;

    // Make SequenceProvider
    // (1) In memory provider
    let mut in_memory_provider = InMemoryProvider::new();
    in_memory_provider.add_fasta_file(fasta_file_path).unwrap();
    let mut in_memory_rc_provider = InMemoryRcProvider::new();
    in_memory_rc_provider.add_fasta_file(fasta_file_path).unwrap();
    // (2) Indexed fasta provider
    let mut indexed_fasta_provider = IndexedFastaProvider::new(fasta_file_path).unwrap();
    let mut indexed_fasta_rc_provider = IndexedFastaRcProvider::new(fasta_file_path).unwrap();

    assert_both_provide_same_sequence(&in_memory_provider, &indexed_fasta_provider);
    assert_both_provide_same_sequence(&in_memory_rc_provider, &indexed_fasta_rc_provider);
    
    assert_both_provide_same_label(&in_memory_provider, &indexed_fasta_provider);
    assert_both_provide_same_label(&in_memory_rc_provider, &indexed_fasta_rc_provider);

    assert_both_provide_same_rc(&in_memory_rc_provider, &indexed_fasta_rc_provider);
}

#[test]
fn test_sequence_providers_serialization() {
    let fasta_file_path = NUCLEOTIDE_ONLY_FA_PATH_1;

    // (1) In memory provider
    {
        let mut in_memory_provider = InMemoryProvider::new();
        in_memory_provider.add_fasta_file(fasta_file_path).unwrap();
        assert_provider_serialization(&in_memory_provider);
    }
    {
        let mut in_memory_rc_provider = InMemoryRcProvider::new();
        in_memory_rc_provider.add_fasta_file(fasta_file_path).unwrap();
        assert_provider_serialization(&in_memory_rc_provider);
    }
    // (2) Indexed fasta provider
    {
        let mut indexed_fasta_provider = IndexedFastaProvider::new(fasta_file_path).unwrap();
        assert_provider_serialization(&indexed_fasta_provider);
    }
    {
        let mut indexed_fasta_rc_provider = IndexedFastaRcProvider::new(fasta_file_path).unwrap();
        assert_provider_serialization(&indexed_fasta_rc_provider);
    }
}

fn assert_both_provide_same_sequence<S1, S2>(
    sequence_provider_1: &S1,
    sequence_provider_2: &S2,
) where
    S1: SequenceProvider,
    S2: SequenceProvider,
{
    let read_count_1 = sequence_provider_1.total_record_count();
    let read_count_2 = sequence_provider_2.total_record_count();

    assert_eq!(read_count_1, read_count_2);

    let mut sequence_buffer_1 = sequence_provider_1.get_buffer();
    let mut sequence_buffer_2 = sequence_provider_2.get_buffer();
    for record_index in 0..read_count_1 {
        sequence_provider_1.fill_sequence_buffer(record_index, &mut sequence_buffer_1);
        sequence_provider_2.fill_sequence_buffer(record_index, &mut sequence_buffer_2);

        let sequence_1 = sequence_buffer_1.request_sequence().to_vec();
        let sequence_2 = sequence_buffer_2.request_sequence().to_vec();

        assert_eq!(sequence_1, sequence_2);
    }
}

fn assert_both_provide_same_label<SL1, SL2>(
    sequence_provider_1: &SL1,
    sequence_provider_2: &SL2,
) where
    SL1: SequenceProvider + LabelProvider,
    SL2: SequenceProvider + LabelProvider,
{
    let read_count_1 = sequence_provider_1.total_record_count();
    let read_count_2 = sequence_provider_2.total_record_count();
    
    assert_eq!(read_count_1, read_count_2);

    for record_index in 0..read_count_1 {
        let label_1 = sequence_provider_1.label_of_record(record_index);
        let label_2 = sequence_provider_2.label_of_record(record_index);

        assert_eq!(label_1, label_2);
    }
}

fn assert_provider_serialization<S>(
    sequence_provider_to_save: &S,
) where
    S: SequenceProvider + Serializable,
{
    // Save
    let mut buffer = Vec::new();
    sequence_provider_to_save.save_to(&mut buffer).unwrap();

    // Load
    let buffer_cursor = Cursor::new(buffer);
    let loaded_sequence_provider = S::load_from(buffer_cursor).unwrap();
    
    // trait 'PatialEq' impl is not guaranteed
    assert_both_provide_same_sequence(sequence_provider_to_save, &loaded_sequence_provider)
}

fn assert_both_provide_same_rc<SR1, SR2>(
    sequence_provider_1: &SR1,
    sequence_provider_2: &SR2,
) where
    SR1: SequenceProvider + ReverseComplement,
    SR2: SequenceProvider + ReverseComplement,
{
    let read_count_1 = sequence_provider_1.total_record_count();
    let read_count_2 = sequence_provider_2.total_record_count();
    
    assert_eq!(read_count_1, read_count_2);

    for record_index in 0..read_count_1 {
        let is_rc_1 = sequence_provider_1.is_reverse_complement(record_index);
        let is_rc_2 = sequence_provider_2.is_reverse_complement(record_index);

        assert_eq!(is_rc_1, is_rc_2);
    }
}
