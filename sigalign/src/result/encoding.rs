use crate::{Result, error_msg};
use crate::{Serialize, Deserialize};
use super::{
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};
use super::{
    FastaAlignmentResult,
    ReadAlignmentResult,
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
};

use std::io::Write;
use serde_json::{
    to_string,
    to_string_pretty,
    to_writer,
    to_writer_pretty,
};

pub fn write_as_json<V, W>(value: &V, writer: W) -> Result<()> where
    V: Serialize,
    W: Write,
{
    match to_writer(writer, value) {
        Ok(_) => Ok(()),
        Err(err) => error_msg!(err),
    }
}
pub fn write_as_json_pretty<V, W>(value: &V, writer: W) -> Result<()> where
    V: Serialize,
    W: Write,
{
    match to_writer_pretty(writer, value) {
        Ok(_) => Ok(()),
        Err(err) => error_msg!(err),
    }
}

impl FastaAlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
impl ReadAlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
impl AlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
impl FastaAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
impl ReadAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
impl AlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json(self, writer)
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) -> Result<()> {
        write_as_json_pretty(self, writer)
    }
}
