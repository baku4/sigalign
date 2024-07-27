use core::fmt;

use super::{
    PyAlignment, PyAlignmentOperation, PyAlignmentOperations, PyFastaAlignment, PyQueryAlignment,
    PyReadAlignment, PyTargetAlignment,
};

impl PyFastaAlignment {
    pub fn py_debug(&self) -> String {
        format!("FastaAlignment(num_read_alignments={})", self.0.len(),)
    }
}

impl PyReadAlignment {
    pub fn py_debug(&self) -> String {
        format!(
            "ReadAlignment(read={}, is_forward={}, num_target_alignments={})",
            self.read,
            self.is_forward,
            self.result.0.len(),
        )
    }
}

impl PyQueryAlignment {
    pub fn py_debug(&self) -> String {
        format!("QueryAlignment(num_target_alignments={})", self.0.len(),)
    }
}

impl PyTargetAlignment {
    pub fn py_debug(&self) -> String {
        format!(
            "TargetAlignment(index={}, label={:?}, num_alignments={})",
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

impl PyAlignmentOperations {
    pub fn py_debug(&self) -> String {
        format!(
            "AlignmentOperations(operation={}, count={})",
            self.operation, self.count
        )
    }
}

impl PyAlignmentOperation {
    pub fn py_debug(&self) -> String {
        format!("{:?}", self)
    }
}
impl fmt::Display for PyAlignmentOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Match => "Match",
                Self::Subst => "Subst",
                Self::Insertion => "Insertion",
                Self::Deletion => "Deletion",
            }
        )
    }
}
