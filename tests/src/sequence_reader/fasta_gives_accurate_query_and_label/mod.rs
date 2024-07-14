use std::fs::File;

use sigalign_utils::sequence_reader::{
    SeqRecord,
    SeqRefRecord,
    IdRecord,
    IdRefRecord,
    fasta::{FastaReader, FastaRecord},
    fastq::{FastqReader, FastqRecord},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Record {
    id: String,
    seq: Vec<u8>,
}

mod read_using_bio_crate;
use read_using_bio_crate::get_record_iterator_from_bio_crate;

use crate::common::test_data::DataForValidation;

#[test]
fn test_read_fasta_file() {
    use crate::common::test_data::DataForValidation;

    let (path_1, path_2) = DataForValidation::Default.get_data_paths();

    for file_path in [path_1, path_2] {
        println!("Reading file: {:?}", file_path);

        // Get Reader from bio crate
        let mut reader_from_bio = get_record_iterator_from_bio_crate(
            &file_path.as_os_str().to_str().unwrap()
        );

        // Get Reader from sigalign-utils
        let file = File::open(&file_path).unwrap();
        let mut reader_from_utils = FastaReader::new(file);

        // Check if they give same results

        let mut query_buffer = Vec::new();
        let mut label_buffer = String::new();
        
        let mut total_records = 0;
        while let Some(mut record) = reader_from_utils.next() {
            total_records += 1;
            query_buffer.clear();
            label_buffer.clear();

            record.extend_seq_buf(&mut query_buffer);
            record.extend_id_string(&mut label_buffer).unwrap();

            let record_from_bio = reader_from_bio.next().unwrap();
            assert_eq!(record_from_bio.id, label_buffer);
            assert_eq!(record_from_bio.seq, query_buffer);
        }
        println!("Total records: {}", total_records);
    }
}
