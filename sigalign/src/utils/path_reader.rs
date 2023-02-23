use std::path::{PathBuf, Path};

pub fn path_to_byte<P>(path: P) -> Result<Vec<u8>> where
    P: AsRef<Path>, // + std::fmt::Debug,
{
    let optional_str = path.as_ref().to_str();
    match optional_str {
        Some(v) => Ok(v.as_bytes().to_vec()),
        None => error_msg!("Path is not convertible to string"),
    }
}

pub fn byte_to_pathbuf(byte: &[u8]) -> Result<PathBuf> {
    let string = String::from_utf8(byte.to_vec())?;
    let pathbuf = PathBuf::from(string);
    Ok(pathbuf)
}
