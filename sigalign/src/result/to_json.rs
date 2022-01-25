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

impl FastaAlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
impl ReadAlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
impl AlignmentResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
impl FastaAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
impl ReadAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
impl AlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    pub fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    pub fn write_as_json<W: Write>(&self, writer: W) {
        to_writer(writer, self).unwrap()
    }
    pub fn write_as_json_pretty<W: Write>(&self, writer: W) {
        to_writer_pretty(writer, self).unwrap()
    }
}
