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
use sigalign::{Reference, ReferenceBuilder};

#[test]
fn test_save_and_load_reference() {
    init_logger();

    let test_data = DataForValidation::Default;
    let (fa_1, fa_2) = test_data.get_data_paths();
    let fasta_file_list = vec![fa_1, fa_2];

    for fasta_file in fasta_file_list {
        info!("Start to test with file: {:?}", fasta_file);
        let original = ReferenceBuilder::new()
            .add_fasta_file(&fasta_file).unwrap()
            .build().unwrap();
        info!("Original reference: {:?}", original);

        // Save and load
        let mut buffer_to_save = Vec::new();
        original.save_to(&mut buffer_to_save).unwrap();
        let loaded = Reference::load_from(&mut buffer_to_save.as_slice()).unwrap();

        // Compare
        let num_targets = {
            let num_targets = original.get_num_targets();
            assert_eq!(num_targets, loaded.get_num_targets());
            num_targets
        };

        for target_index in 0..num_targets {
            let seq_1 = original.get_sequence(target_index).unwrap();
            let seq_2 = loaded.get_sequence(target_index).unwrap();
            assert_eq!(seq_1, seq_2);

            let label_1 = original.get_label(target_index).unwrap();
            let label_2 = loaded.get_label(target_index).unwrap();
            assert_eq!(label_1, label_2);
        }
    }
}
