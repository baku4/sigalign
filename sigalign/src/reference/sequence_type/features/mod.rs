use super::SequenceType;
use super::super::features::{
    Serialize,
};

use capwriter::{Save, Load};
use std::io::{Write, Read, Error};
impl Serialize for SequenceType {
    fn save_to<W>(&self, mut writer: W) -> Result<(), Error> where
        W: Write
    {
        self.valid_characters().save_to(writer)
    }
    fn load_from<R>(mut reader: R) -> Result<Self, Error> where
        R: Read,
        Self: Sized
    {
        let sequence: Vec<u8> = Vec::load_from(reader)?;
        Ok(Self::new(&sequence))
    }
}