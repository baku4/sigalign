use super::{Result, error_msg};
use std::{env, path::PathBuf, fs};

// For reference building
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

// For result validation
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

// For data caching
const LOCAL_TMP_DIR: &str = "tmp";
pub fn get_local_tmp_dir() -> Result<PathBuf> {
    // let current_dir = env::current_dir()?;
    // let local_tmp_dir = current_dir.join(LOCAL_TMP_DIR);

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
