use wasm_bindgen::prelude::*;

use sigalign::results::{
    labeled::{
        LabeledAlignmentResult,
    },
};

#[wasm_bindgen]
pub struct AlignmentResult(LabeledAlignmentResult);

#[wasm_bindgen]
impl AlignmentResult {
    pub(crate) fn new(inner_result: LabeledAlignmentResult) -> Self {
        Self(inner_result)
    }
    pub fn to_json(&self) -> String {
        self.0.to_json()
    }
    pub fn to_table(&self) -> Vec<JsValue> {
        let result_count = self.0.result_counts();
        let mut table: Vec<JsValue> = Vec::with_capacity(result_count);

        self.0.0.iter().for_each(| target_result | {
            target_result.alignments.iter().for_each(| anchor_result | {
                let row: Row = Row {
                    index: target_result.index,
                    label: target_result.label.clone(),
                    penalty: anchor_result.penalty,
                    length: anchor_result.length,
                    qstart: anchor_result.position.query.0,
                    qend: anchor_result.position.query.1,
                    rstart: anchor_result.position.target.0,
                    rend: anchor_result.position.target.1,
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
    pub index: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub label: String,
    pub penalty: u32,
    pub length: u32,
    pub qstart: u32,
    pub qend: u32,
    pub rstart: u32,
    pub rend: u32,
    #[wasm_bindgen(getter_with_clone)]
    pub ops: String,
}

use sigalign::results::{
    AlignmentOperations,
    AlignmentOperation,
};
fn operations_to_string(operations: &Vec<AlignmentOperations>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|ops| {
        format!(
            "{}{}",
            match ops.operation {
                AlignmentOperation::Match => 'M',
                AlignmentOperation::Subst => 'S',
                AlignmentOperation::Insertion => 'I',
                AlignmentOperation::Deletion => 'D',
            },
            ops.count,
        )
    }).collect();
    string_ops.concat()
}
