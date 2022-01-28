use crate::{Result, error_msg};

pub const NUCLEOTIDE_ONLY_FA_PATH_1: &str = "../sample_data/nucleotide_only/ERR209055.fa";
pub const NUCLEOTIDE_ONLY_FA_PATH_2: &str = "../sample_data/nucleotide_only/ERR209056.fa";

pub const FA_ENDS_WITH_CRLF_PATH: &str = "../sample_data/nucleotide_only/ERR209055_crlf.fa";
pub const SIMPLE_FA_PATH: &str = "../sample_data/nucleotide_only/ERR209055_simple.fa";

use std::env;
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write, Read, BufRead};

use flate2::read::GzDecoder;

// Reference 1:
// Mycobacterium Tuberculosis - H37Rv
const REFERENCE_1_NAME: &str = "H37Rv.fa";
const REFERENCE_1_FTP_URL: &str = "https://ftp.ncbi.nlm.nih.gov/genomes/all/GCF/000/195/955/GCF_000195955.2_ASM19595v2/GCF_000195955.2_ASM19595v2_genomic.fna.gz";

pub fn prepare_reference_1() -> PathBuf {
    let sample_data_dir = get_sample_data_dir().unwrap();
    
    let reference_path = sample_data_dir.join(REFERENCE_1_NAME);

    if !reference_path.exists() {
        // Download original data
        let raw_file_name = REFERENCE_1_FTP_URL.rsplit("/").next().unwrap();
        let raw_file_path = sample_data_dir.join(raw_file_name);
        let mut raw_file = if raw_file_path.exists() {
            let mut raw_file = fs::File::open(&raw_file_path).unwrap();
            raw_file
        } else {
            let mut raw_file = fs::File::create(&raw_file_path).unwrap();
            let mut resp = reqwest::blocking::get(REFERENCE_1_FTP_URL).unwrap();
            resp.copy_to(&mut raw_file).unwrap();
            raw_file
        };

        // Decompress
        let mut gz_decoder = GzDecoder::new(raw_file);
        let mut gz_buf_reader = io::BufReader::new(gz_decoder);

        let mut out = fs::File::create(&reference_path).unwrap();

        io::copy(&mut gz_buf_reader, &mut out).unwrap();
    }

    reference_path
}

const SAMPLE_DATA_DIR_NAME: &str = "samples";

fn get_sample_data_dir() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    
    let sample_data_dir = current_dir.join(SAMPLE_DATA_DIR_NAME);

    if sample_data_dir.exists() {
        if !sample_data_dir.is_dir() {
            error_msg!("Sample data path is not directory")
        }
    } else {
        fs::create_dir(&sample_data_dir)?;
    }

    Ok(sample_data_dir)
}