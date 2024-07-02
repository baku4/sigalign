use super::Record;
use bio::io::fasta;
use std::fs::File;
use std::io::BufReader;

pub fn get_record_iterator_from_bio_crate(file_path: &str) -> Box<dyn Iterator<Item = Record>> {
    let file = File::open(file_path).unwrap();
    let reader = fasta::Reader::new(BufReader::new(file));

    let iter = reader.records().map(|record| {
        let r = record.unwrap();
        Record {
            id: r.id().to_string(),
            seq: r.seq().to_vec(),
        }
    });

    Box::new(iter)
}
