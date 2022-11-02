use super::{
    Result,
};

use std::io::{Write, Read};

/// Save and Load
pub trait Serializable {
    fn save_to<W>(&self, writer: W) -> Result<()> where W: Write;
    fn load_from<R>(reader: R) -> Result<Self> where R: Read, Self: Sized;
}

/// Precalculate saved size
pub trait SizeAware: Serializable {
    fn size_of(&self) -> usize;
}
