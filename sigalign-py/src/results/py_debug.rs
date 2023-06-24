use super::{
    FastaResult,
    PyReadResult,
    QueryResult,
    PyTargetResult,
    PyAlignment,
    PyOperation,
};

impl FastaResult {
    pub fn py_debug(&self) -> String {
        format!(
            "FastaResult(num_read_results={})",
            self.0.len(),
        )
    }
}

impl PyReadResult {
    pub fn py_debug(&self) -> String {
        format!(
            "ReadResult(read={}, num_target_results={})",
            self.read,
            self.result.0.len(),
        )
    }
}

impl QueryResult {
    pub fn py_debug(&self) -> String {
        format!(
            "QueryResult(num_target_results={})",
            self.0.len(),
        )
    }
}

impl PyTargetResult {
    pub fn py_debug(&self) -> String {
        format!(
            "TargetResult(index={}, label={:?}, num_alignments={})",
            self.index,
            self.label,
            self.num_alignments(),
        )
    }
}

impl PyAlignment {
    pub fn py_debug(&self) -> String {
        format!(
            "PyAlignment(penalty={}, length={}, query_position={:?}, target_position={:?}, num_operations={:?})",
            self.penalty,
            self.length,
            self.query_position,
            self.target_position,
            self.operations.len(),
        )
    }
}

impl PyOperation {
    pub fn py_debug(&self) -> String {
        format!("Operation(case={}, count={})", self.case, self.count)
    }
}
