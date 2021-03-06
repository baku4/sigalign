use crate::{Result, error_msg};
use crate::{Serialize, DeserializeOwned};

use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};

pub trait Writable: Serialize + DeserializeOwned {
    /// Write to file
    fn write_to_file<P: AsRef<Path>>(&self, file_path: P) -> Result<()> {
        let file = {
            match File::create(file_path) {
                Ok(file) => file,
                Err(err) => error_msg!("{}", err),
            }
        };
        self.write_to(file)
    }
    /// Read from file
    fn read_from_file<P: AsRef<Path>>(file_path: P) -> Result<Self> {
        let file = {
            match File::open(file_path) {
                Ok(file) => file,
                Err(err) => error_msg!("{}", err),
            }
        };
        Self::read_from(file)
    }
    /// Write to [Write]r
    fn write_to<W>(&self, writer: W) -> Result<()>
        where W: Write 
    {
        match bincode::serialize_into(writer, self) {
            Ok(_) => Ok(()),
            Err(err) => {
                error_msg!("{}", err)
            },
        }
    }
    /// Read from [Read]r
    fn read_from<R>(reader: R) -> Result<Self>
        where R: Read,
    {
        match bincode::deserialize_from::<R, Self>(reader) {
            Ok(v) => {
                Ok(v)
            },
            Err(err) => {
                error_msg!("{}", err)
            },
        }
    }
}