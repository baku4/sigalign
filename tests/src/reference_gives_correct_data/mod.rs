use crate::common::{
    init_logger,
    test_data::{DataForValidation, DataForRefBuild},
};

use log::info;
use sigalign_utils::sequence_reader::{
    fasta::FastaReader,
    SeqRecord as _,
    IdRecord as _,
};
use sigalign::ReferenceBuilder;

#[test]
fn test_reference_gives_correct_data() {
    init_logger();

    let test_data = DataForValidation::Default;
    let (fa_1, fa_2) = test_data.get_data_paths();
    let fasta_file_list = vec![fa_1, fa_2];

    for fasta_file in fasta_file_list {
        info!("Start to test with file: {:?}", fasta_file);
        let reference_from_fasta_file = ReferenceBuilder::new()
            .add_fasta_file(&fasta_file).unwrap()
            .build().unwrap();

        let reference_from_manual_added_sequence = {
            let mut reference_builder = ReferenceBuilder::new();

            let mut seq_buffer = Vec::new();
            let mut label_buffer = String::new();
            let mut fasta_reader = FastaReader::from_path(&fasta_file).unwrap();

            while let Some(mut record) = fasta_reader.next() {
                seq_buffer.clear();
                label_buffer.clear();

                record.extend_seq_buf(&mut seq_buffer);
                record.extend_id_string(&mut label_buffer).unwrap();

                reference_builder = reference_builder.add_target(
                    &label_buffer,
                    &seq_buffer,
                );
            }

            reference_builder.build().unwrap()
        };

        // Compare results
        info!("Compare results");
        let mut seq_buffer = Vec::new();
        let mut label_buffer = String::new();
        let mut fasta_reader = FastaReader::from_path(&fasta_file).unwrap();

        let mut target_index = 0;
        while let Some(mut record) = fasta_reader.next() {
            seq_buffer.clear();
            label_buffer.clear();

            record.extend_seq_buf(&mut seq_buffer);
            record.extend_id_string(&mut label_buffer).unwrap();

            // Compare sequence
            let seq_from_fasta = reference_from_fasta_file.get_sequence(target_index).unwrap();
            let seq_from_manual = reference_from_manual_added_sequence.get_sequence(target_index).unwrap();
            assert_eq!(seq_from_fasta, seq_buffer);
            assert_eq!(seq_from_manual, seq_buffer);

            // Compare label
            let label_from_fasta = reference_from_fasta_file.get_label(target_index).unwrap();
            let label_from_manual = reference_from_manual_added_sequence.get_label(target_index).unwrap();
            assert_eq!(label_from_fasta, label_buffer);
            assert_eq!(label_from_manual, label_buffer);

            target_index += 1;
        }
    }
}
