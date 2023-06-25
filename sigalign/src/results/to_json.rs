use std::io::{Write, Error};
use serde::Deserialize;
use serde_json::{
    to_string,
    to_string_pretty,
    to_writer,
    to_writer_pretty,
    from_str,
};

fn translate_str_to_result<'a, T>(s: &'a str) -> Result<T, Error> where
    T: Deserialize<'a>
{
    match from_str(s) {
        Ok(v) => Ok(v),
        Err(_) => Err(std::io::ErrorKind::InvalidData.into()),
    }
}

macro_rules! impl_translate_between_json {
    ( $st: ident ) => {
        impl $st {
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
            pub fn from_json(s: &str) -> Result<Self, Error> {
                translate_str_to_result(s)
            }
        }
    };
}

use super::{
    AlignmentResult,
    TargetAlignmentResult,
    AnchorAlignmentResult,
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
        FastaReverseComplementAlignmentResult,
        ReadReverseComplementAlignmentResult,
    },
    labeled::{
        LabeledAlignmentResult,
        LabeledTargetAlignmentResult,
    },
};
impl_translate_between_json!(AlignmentResult);
impl_translate_between_json!(TargetAlignmentResult);
impl_translate_between_json!(AnchorAlignmentResult);
impl_translate_between_json!(FastaAlignmentResult);
impl_translate_between_json!(ReadAlignmentResult);
impl_translate_between_json!(FastaReverseComplementAlignmentResult);
impl_translate_between_json!(ReadReverseComplementAlignmentResult);
impl_translate_between_json!(LabeledAlignmentResult);
impl_translate_between_json!(LabeledTargetAlignmentResult);
