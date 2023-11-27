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
const REF_FILE: &str = "reference.fa";
const QRY_FILE: &str = "query.fa";
pub fn get_ref_for_val_path() -> PathBuf {
    let mut path = PathBuf::from(VALIDATE_RES_DIR);
    path.push(REF_FILE);
    path
}
pub fn get_qry_for_val_path() -> PathBuf {
    let mut path = PathBuf::from(VALIDATE_RES_DIR);
    path.push(QRY_FILE);
    path
}

//
// (3) For data caching
//
const LOCAL_TMP_DIR: &str = "tmp";
pub fn get_local_tmp_dir() -> Result<PathBuf> {
    let local_tmp_dir = PathBuf::from(LOCAL_TMP_DIR);

    if local_tmp_dir.exists() {
        if !local_tmp_dir.is_dir() {
            error_msg!("Local tmp path {:?} is not directory", local_tmp_dir)
        }
    } else {
        fs::create_dir(&local_tmp_dir)?;
    }

    Ok(local_tmp_dir)
}
pub fn get_dir_on_tmp_dir(dir_name: &str) -> Result<PathBuf> {
    let mut path = get_local_tmp_dir()?;
    path.push(dir_name);
    if path.exists() {
        if !path.is_dir() {
            error_msg!("Local tmp path {:?} is not directory", path)
        }
    } else {
        fs::create_dir(&path)?;
    }
    Ok(path)
}
