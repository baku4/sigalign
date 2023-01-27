use wasm_bindgen::prelude::*;

use super::{
    AlignmentLabeledResult,
};

#[wasm_bindgen]
pub struct AlignmentResult(AlignmentLabeledResult);

#[wasm_bindgen]
impl AlignmentResult {
    pub(crate) fn new(inner_result: AlignmentLabeledResult) -> Self {
        Self(inner_result)
    }
    pub fn to_json(&self) -> String {
        self.0.to_json()
    }
    pub fn to_table(&self) -> Vec<JsValue> {
        let result_count = self.0.result_counts();
        let mut table: Vec<JsValue> = Vec::with_capacity(result_count);

        self.0.0.iter().for_each(| record_result | {
            record_result.alignments.iter().for_each(| anchor_result | {
                let row: Row = Row {
                    index: record_result.index,
                    label: record_result.label.clone(),
                    penalty: anchor_result.penalty,
                    length: anchor_result.length,
                    qstart: anchor_result.position.query.0,
                    qend: anchor_result.position.query.1,
                    rstart: anchor_result.position.record.0,
                    rend: anchor_result.position.record.1,
                    ops: operations_to_string(&anchor_result.operations),
                };
                table.push(JsValue::from(row));
            });
        });

        table
    }
}

#[wasm_bindgen]
pub struct Row {
    pub index: usize,
    #[wasm_bindgen(getter_with_clone)]
    pub label: String,
    pub penalty: usize,
    pub length: usize,
    pub qstart: usize,
    pub qend: usize,
    pub rstart: usize,
    pub rend: usize,
    #[wasm_bindgen(getter_with_clone)]
    pub ops: String,
}

use sigalign::core::{
    AlignmentOperation,
    AlignmentCase,
};
fn operations_to_string(operations: &Vec<AlignmentOperation>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.case {
                AlignmentCase::Match => 'M',
                AlignmentCase::Subst => 'S',
                AlignmentCase::Insertion => 'I',
                AlignmentCase::Deletion => 'D',
            },
            op.count,
        )
    }).collect();
    string_ops.concat()
}
