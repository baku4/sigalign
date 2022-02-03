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
};

impl FastaAlignmentResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}

impl ReadAlignmentResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}
impl AlignmentResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}
impl FastaAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}
impl ReadAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}
impl AlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}
impl RecordAlignmentLabeledResult {
    pub fn to_json(&self) -> String {
        self.json_value()
    }
    pub fn to_json_short(&self) -> String {
        self.json_value_short()
    }
    pub fn write_as_json<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json().as_bytes()).unwrap();
    }
    pub fn write_as_json_short<W: Write>(&self, mut writer: W) {
        writer.write_all(self.to_json_short().as_bytes()).unwrap();
    }
}

impl JsonValue for FastaAlignmentLabeledResult {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value()).collect();
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value_short()).collect();
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for FastaAlignmentResult {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value()).collect();
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value_short()).collect();
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for ReadAlignmentLabeledResult {
    fn json_value(&self) -> String {
        format!("{{\"read\":{},\"result\":{}}}", self.read, self.result.json_value())
    }
    fn json_value_short(&self) -> String {
        format!("{{\"rd\":{},\"res\":{}}}", self.read, self.result.json_value_short())
    }
}

impl JsonValue for ReadAlignmentResult {
    fn json_value(&self) -> String {
        format!("{{\"read\":{},\"result\":{}}}", self.read, self.result.json_value())
    }
    fn json_value_short(&self) -> String {
        format!("{{\"rd\":{},\"res\":{}}}", self.read, self.result.json_value_short())
    }
}

impl JsonValue for AlignmentLabeledResult {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value()).collect();
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value_short()).collect();
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for AlignmentResult {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value()).collect();
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.0.iter().map(|v| v.json_value_short()).collect();
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for RecordAlignmentLabeledResult {
    fn json_value(&self) -> String {
        format!("{{\"index\":{},\"label\":{},\"result\":{}}}", self.index, self.label, self.result.json_value())
    }
    fn json_value_short(&self) -> String {
        format!("{{\"idx\":{},\"lbl\":{},\"res\":{}}}", self.index, self.label, self.result.json_value_short())
    }
}

impl JsonValue for RecordAlignmentResult {
    fn json_value(&self) -> String {
        format!("{{\"index\":{},\"result\":{}}}", self.index, self.result.json_value())
    }
    fn json_value_short(&self) -> String {
        format!("{{\"idx\":{},\"res\":{}}}", self.index, self.result.json_value_short())
    }
}

impl JsonValue for Vec<AnchorAlignmentResult> {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.iter().map(|v| v.json_value()).collect();
        
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.iter().map(|v| v.json_value_short()).collect();
        
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for AnchorAlignmentResult {
    fn json_value(&self) -> String {
        format!("{{\"penalty\":{},\"length\":{},\"position\":{},\"operations\":{}}}", self.penalty, self.length, self.position.json_value(), self.operations.json_value())
    }
    fn json_value_short(&self) -> String {
        format!("{{\"p\":{},\"l\":{},\"pos\":{},\"ops\":{}}}", self.penalty, self.length, self.position.json_value_short(), self.operations.json_value_short())
    }
}

impl JsonValue for AlignmentPosition {
    fn json_value(&self) -> String {
        format!("{{\"record\":[{},{}],\"query\":[{},{}]}}", self.record.0, self.record.1, self.query.0, self.query.0)
    }
    fn json_value_short(&self) -> String {
        format!("{{\"r\":[{},{}],\"q\":[{},{}]}}", self.record.0, self.record.1, self.query.0, self.query.0)
    }
}

impl JsonValue for Vec<AlignmentOperation> {
    fn json_value(&self) -> String {
        let list: Vec<String> = self.iter().map(|v| v.json_value()).collect();
        
        format!("[{}]", list.join(","))
    }
    fn json_value_short(&self) -> String {
        let list: Vec<String> = self.iter().map(|v| v.json_value_short()).collect();
        
        format!("[{}]", list.join(","))
    }
}

impl JsonValue for AlignmentOperation {
    fn json_value(&self) -> String {
        format!("{{\"case\":{case},\"count\":{count}}}", case=self.case.symbol(), count=self.count)
    }
    fn json_value_short(&self) -> String {
        format!("{{\"c\":{case},\"n\":{count}}}", case=self.case.symbol_short(), count=self.count)
    }
}

impl AlignmentCase {
    fn symbol(&self) -> &str {
        match self {
            Self::Match => "Match",
            Self::Subst => "Subst",
            Self::Insertion => "Ins",
            Self::Deletion => "Del",
        }
    }
    fn symbol_short(&self) -> &str {
        match self {
            Self::Match => "M",
            Self::Subst => "S",
            Self::Insertion => "I",
            Self::Deletion => "D",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn print_json_of_struct() {
        // AlignmentOperation
        println!("AlignmentOperation");
        let alignment_operation = AlignmentOperation {
            case: AlignmentCase::Match,
            count: 100,
        };

        println!("{:?}", alignment_operation.json_value());
        println!("{:?}", alignment_operation.json_value_short());

        // AlignmentPosition
        println!("AlignmentPosition");
        let alignment_position = AlignmentPosition {
            record: (0, 150),
            query: (20, 50),
        };

        println!("{:?}", alignment_position.json_value());
        println!("{:?}", alignment_position.json_value_short());

        // AnchorAlignmentResult
        println!("AnchorAlignmentResult");
        let anchor_alignment_result = AnchorAlignmentResult {
            penalty: 10,
            length: 150,
            position: alignment_position,
            operations: vec![alignment_operation; 3],
        };

        println!("{:?}", anchor_alignment_result.json_value());
        println!("{:?}", anchor_alignment_result.json_value_short());
    }
}