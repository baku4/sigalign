use std::io::{Write, Read, Error};

/// Save and Load
pub trait Serialize {
    fn save_to<W>(&self, writer: W) -> Result<(), Error> where W: Write;
    fn load_from<R>(reader: R) -> Result<Self, Error> where R: Read, Self: Sized;
}

/// Precalculate saved size
pub trait EstimateSize: Serialize {
    fn size_of(&self) -> usize;
}
