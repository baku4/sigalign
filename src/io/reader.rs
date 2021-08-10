use std::path::Path;
use std::{fs::read, io::BufReader};
use std::fs::File;

// Lt Fm Index
use lt_fm_index::FmIndex;

const LT_FMI_EXT: &str = "lfi";

pub fn read_lt_fm_index_from_file_path(file_path: &str) -> Result<FmIndex, String> {
    FmIndex::read_index_from_file(file_path)
}
pub fn read_lt_fm_index_from_infered_path(org_file_path: &str) -> Result<FmIndex, String> {
    let path = Path::new(org_file_path);
    let dir = path.parent().unwrap();
    let file_name = path.file_name().unwrap();
    // TODO: Split once
    let first_stem = file_name.to_str().unwrap().to_string().split('.').into_iter().next().unwrap();
    let fm_index_path = dir.join(format!("{}.{}", first_stem, LT_FMI_EXT)).into_os_string().into_string().unwrap();
    FmIndex::read_index_from_file(&fm_index_path)
}

/*  FASTA & FASTAQ  */
use bio::io::fasta;
// From `Rust-Bio` crate  
// https://crates.io/crates/bio  
pub type FastaReader = fasta::Reader<BufReader<File>>;
pub type FastaRecords = fasta::Records<BufReader<File>>;
pub fn fasta_records(file_path: &str) -> FastaRecords {
    fasta::Reader::from_file(file_path).expect("Errored in reading fasta").records()
}
