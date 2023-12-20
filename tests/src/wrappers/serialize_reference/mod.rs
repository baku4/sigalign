use log::info;
use sigalign::{Reference, ReferenceBuilder};

use crate::common::{
    init_logger,
    test_data_path::{
        get_ref_for_val_path,
        get_qry_for_val_path,
        get_dir_on_tmp_dir,
    },
};

#[test]
fn test_reference_serialize() {
    init_logger();

    let fa_file_1 = get_ref_for_val_path();
    let fa_file_2 = get_qry_for_val_path();

    let mut buffer = Vec::new();

    for fa_file in [fa_file_1, fa_file_2].iter() {
        info!("fa_file: {:?}", fa_file);
        let to_save_ref = ReferenceBuilder::new().add_fasta_file(&fa_file).unwrap().build().unwrap();

        buffer.clear();
        to_save_ref.save_to(&mut buffer).unwrap();

        let loaded_ref = Reference::load_from(buffer.as_slice()).unwrap();

        let num_targets_1 = to_save_ref.get_num_targets();
        let num_targets_2 = loaded_ref.get_num_targets();
        assert_eq!(num_targets_1, num_targets_2);

        info!("num_targets: {}", num_targets_1);
        for i in 0..to_save_ref.get_num_targets() {
            let seq_1 = to_save_ref.get_sequence(i).unwrap();
            let seq_2 = loaded_ref.get_sequence(i).unwrap();
            assert_eq!(seq_1, seq_2);

            let label_1 = to_save_ref.get_label(i).unwrap();
            let label_2 = loaded_ref.get_label(i).unwrap();
            assert_eq!(label_1, label_2);
        }
    }
}