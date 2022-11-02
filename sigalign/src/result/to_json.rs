use crate::{Result, error_msg};
use crate::Deserialize;
use super::{
    AlignmentResult,
};
use super::{
    FastaAlignmentResult,
    ReadAlignmentResult,
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
};

trait JsonValue {
    fn json_value(&self) -> String;
    fn json_value_short(&self) -> String;
}

use std::io::Write;
use serde_json::{
    to_string,
    to_string_pretty,
    to_writer,
    to_writer_pretty,
    from_str,
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
    }
}
impl RecordAlignmentLabeledResult {
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
    pub fn from_json(s: &str) -> Result<Self> {
        translate_str_to_result(s)
    }
}

fn translate_str_to_result<'a, T>(s: &'a str) -> Result<T> where
    T: Deserialize<'a>
{
    match from_str(s) {
        Ok(v) => Ok(v),
        Err(e) => error_msg!("Str to result translation failed. {}", e),
    }
}
