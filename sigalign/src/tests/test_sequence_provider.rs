use super::*;

#[test]
fn test_sequence_providers_provide_same_data() {
    let fasta_file_path = NUCLEOTIDE_ONLY_FA_PATH_1;

    // Making SequenceProvider
    let mut in_memory_provider = InMemoryProvider::from_fasta_file(fasta_file_path).unwrap();

    let mut indexed_fasta_provider = IndexedFastaProvider::new(fasta_file_path).unwrap();

    let mut sqlite_provider = SqliteProvider::new_with_in_memory_db_from_fasta(fasta_file_path).unwrap();

    // Record count
    let record_count = in_memory_provider.total_record_count();

    assert_eq!(record_count, indexed_fasta_provider.total_record_count());
    assert_eq!(record_count, sqlite_provider.total_record_count());

    // Sequence and Label
    for record_index in 0..record_count {
        let sequence_from_in_memory_provider = in_memory_provider.sequence_of_record(record_index);
        let sequence_from_indexed_fasta_provider = indexed_fasta_provider.sequence_of_record(record_index);
        let sequence_from_sqlite_provider = sqlite_provider.sequence_of_record(record_index);

        assert_eq!(sequence_from_in_memory_provider, sequence_from_indexed_fasta_provider);
        assert_eq!(sequence_from_in_memory_provider, sequence_from_sqlite_provider);

        let label_from_in_memory_provider = in_memory_provider.label_of_record(record_index);
        let label_from_sqlite_provider = sqlite_provider.label_of_record(record_index);

        assert_eq!(label_from_in_memory_provider, label_from_sqlite_provider);
    }
}