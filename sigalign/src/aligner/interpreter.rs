use crate::{Result, error_msg};
use super::{AlignmentResultsByRecordIndex, AlignmentResultsWithLabelByRecordIndex, AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};
use super::{Reference, SequenceProvider, Labeling};

use serde::{Deserialize, Serialize};
use serde_json::to_string as serialize_to_string;

use std::collections::HashMap;

pub fn raw_result_to_json<S: Serialize>(raw: S) -> Result<String> {
    match serialize_to_string(&raw) {
        Ok(json) => Ok(json),
        Err(error) => error_msg!("{}", error),
    }
}

impl AlignmentResultsByRecordIndex {
    pub fn to_labeled_results<SL: SequenceProvider + Labeling>(
        self,
        reference: &mut Reference<SL>,
    ) -> AlignmentResultsWithLabelByRecordIndex {
        AlignmentResultsWithLabelByRecordIndex(
            self.0.into_iter().map(|(record_index, alignment_results)| {
                let label = reference.label_of_record(record_index).to_string();
                (record_index, (label, alignment_results))
            }).collect()
        )
    }
}
