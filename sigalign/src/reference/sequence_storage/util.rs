use super::{
    Result, error_msg,
};

use std::path::Path;

pub fn path_to_string<P>(file_path: P) -> Result<String> where
    P: AsRef<Path> + std::fmt::Debug,
{
    match file_path.as_ref().canonicalize()?.to_str() {
        Some(v) => Ok(v.to_string()),
        None => error_msg!("Invalid file path"),
    }
}
