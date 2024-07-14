use super::{Result, error_msg};
use std::io::{Read, Write};
use std::{env, path::PathBuf, fs};

// 
// (1) For reference building
//
const BUILD_REF_DIR: &str = "test_data/build_reference";
const LF_FILE: &str = "LF.fa";
const CRLF_FILE: &str = "CRLF.fa";
const TWO_LINE_FILE: &str = "two_line.fa";
pub fn get_lf_fa_path() -> PathBuf {
    let mut path = PathBuf::from(BUILD_REF_DIR);
    path.push(LF_FILE);
    path
}
pub fn get_crlf_fa_path() -> PathBuf {
    let mut path = PathBuf::from(BUILD_REF_DIR);
    path.push(CRLF_FILE);
    path
}
pub fn get_two_line_fa_path() -> PathBuf {
    let mut path = PathBuf::from(BUILD_REF_DIR);
    path.push(TWO_LINE_FILE);
    path
}
pub fn get_gzip_compressed_lf_fa_path() -> PathBuf {
    let mut path = get_lf_fa_path();
    path.set_extension("fa.gz");
    path
}
pub fn get_zlib_compressed_lf_fa_path() -> PathBuf {
    let mut path = get_lf_fa_path();
    path.set_extension("fa.zlib");
    path
}

// 
// (2) For result validation
//
const VALIDATE_RES_DIR: &str = "test_data/validate_result";
#[derive(Clone, Debug)]
pub enum DataForValidation {
    Default,
    OneData,
}
impl DataForValidation {
    /// Return reference and query file paths
    pub fn get_data_paths(&self) -> (PathBuf, PathBuf) {
        let mut path = PathBuf::from(VALIDATE_RES_DIR);
        path.push(self.dir_name());
        let (ref_fname, qry_fname) = self.file_name();
        (
            path.join(ref_fname),
            path.join(qry_fname),
        )
    }
    /// Get unique tag for each dataset
    pub fn get_tag(&self) -> &str {
        match self {
            DataForValidation::Default => "val_def",
            DataForValidation::OneData => "val_one",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag {
            "val_def" => Ok(DataForValidation::Default),
            "val_one" => Ok(DataForValidation::OneData),
            _ => error_msg!("Unknown tag: {}", tag),
        }
    }
    fn dir_name(&self) -> &str {
        match self {
            DataForValidation::Default => "default",
            DataForValidation::OneData => "one_data",
        }
    }
    fn file_name(&self) -> (&str, &str) {
        match self {
            DataForValidation::Default => ("reference.fa", "query.fa"),
            DataForValidation::OneData => ("reference.fa", "query.fa"),
        }
    }
}
