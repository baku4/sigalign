use super::SequenceType;
use super::super::extensions::{
    Serialize,
};

use capwriter::{Save, Load};
use std::io::{Write, Read, Error};
impl Serialize for SequenceType {
    fn save_to<W>(&self, writer: W) -> Result<(), Error> where
        W: Write
    {
        self.alignable_sequence().save_to(writer)
    }
    fn load_from<R>(reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized
    {
        let sequence: Vec<u8> = Vec::load_from(reader)?;
        Ok(Self::infer_from_sequence(&sequence))
    }
}