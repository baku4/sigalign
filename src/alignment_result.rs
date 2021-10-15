use crate::{Result, error_msg};

use crate::core::{AlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentType};

use serde::Serialize;

pub fn to_json<R: Serialize>(result: &R) -> String {
    let json = serde_json::to_string(result).unwrap();
    json
}
