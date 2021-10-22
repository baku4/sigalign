use crate::core::{AlignmentResultsByRecord, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};

use serde_json;

struct Interpreter {}

impl Interpreter {
    fn record_index_to_label() {
        
    }
    fn result_to_json(alignment_results_by_record: &AlignmentResultsByRecord) {
        let json = serde_json::to_string(&alignment_results_by_record).unwrap();
    }
}