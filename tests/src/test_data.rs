use std::{env, path::PathBuf, fs};
use super::{Result, error_msg};

pub const BUILD_REF_DIR: &str = "../test_data/build_reference";
pub const VALIDATE_RES_DIR: &str = "../test_data/validate_result";



fn get_sample_data_dir() -> Result<PathBuf> {
    let current_dir = env::current_dir()?;
    
    let sample_data_dir = current_dir.join(BUILD_REF_DIR);

    if sample_data_dir.exists() {
        if !sample_data_dir.is_dir() {
            error_msg!("Sample data path is not directory")
        }
    } else {
        fs::create_dir(&sample_data_dir)?;
    }

    Ok(sample_data_dir)
}