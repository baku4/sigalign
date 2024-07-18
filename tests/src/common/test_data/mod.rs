use super::{Result, error_msg};
use std::io::{Read, Write};
use std::{env, path::PathBuf, fs};

// 
// (1) For reference building
//
const BUILD_REF_DIR: &str = "test_data/build_reference";
#[derive(Clone, Debug)]
pub enum DataForRefBuild {
    LF,
    CRLF,
    TwoLine,
    Gz,
    Zlib,
}
impl DataForRefBuild {
    /// Return reference and query file paths
    pub fn get_data_path(&self) -> PathBuf {
        let mut path = PathBuf::from(BUILD_REF_DIR);
        path.push(self.file_name());
        path
    }
    /// Get unique tag for each dataset
    pub fn get_tag(&self) -> &str {
        match self {
            Self::LF => "lf",
            Self::CRLF => "crlf",
            Self::TwoLine => "two_line",
            Self::Gz => "gz",
            Self::Zlib => "zlib",
        }
    }
    pub fn from_tag(tag: &str) -> Result<Self> {
        match tag {
            "lf" => Ok(Self::LF),
            "crlf" => Ok(Self::CRLF),
            "two_line" => Ok(Self::TwoLine),
            "gz" => Ok(Self::Gz),
            "zlib" => Ok(Self::Zlib),
            _ => error_msg!("Unknown tag: {}", tag),
        }
    }
    fn file_name(&self) -> &str {
        match self {
            Self::LF => "LF.fa",
            Self::CRLF => "CRLF.fa",
            Self::TwoLine => "two_line.fa",
            Self::Gz => "LF.fa.gz",
            Self::Zlib => "LF.fa.zlib",
        }
    }
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
