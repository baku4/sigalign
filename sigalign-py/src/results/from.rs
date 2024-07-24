use sigalign::results::{
    QueryAlignment, LabeledQueryAlignment, TargetAlignment, LabeledTargetAlignment,
    Alignment, AlignmentOperations, AlignmentOperation,
};

use super::{
    PyQueryAlignment,
    PyTargetAlignment,
    PyAlignment,
    PyAlignmentOperations,
    PyAlignmentOperation,
};

impl From<QueryAlignment> for PyQueryAlignment {
    fn from(query_alignment: QueryAlignment) -> Self {
        Self(query_alignment.0.into_iter().map(|x| {
            PyTargetAlignment::from(x)
        }).collect())
    }
}

impl From<LabeledQueryAlignment> for PyQueryAlignment {
    fn from(labeled_query_alignment: LabeledQueryAlignment) -> Self {
        Self(labeled_query_alignment.0.into_iter().map(|x| {
            PyTargetAlignment::from(x)
        }).collect())
    }
}

impl From<TargetAlignment> for PyTargetAlignment {
    fn from(target_alignment: TargetAlignment) -> Self {
        Self {
            index: target_alignment.index,
            label: None,
            alignments: target_alignment.alignments.into_iter().map(|v| {
                PyAlignment::from(v)
            }).collect(),
        }
    }
}

impl From<LabeledTargetAlignment> for PyTargetAlignment {
    fn from(labeled_target_alignment: LabeledTargetAlignment) -> Self {
        Self {
            index: labeled_target_alignment.index,
            label: Some(labeled_target_alignment.label),
            alignments: labeled_target_alignment.alignments.into_iter().map(|v| {
                PyAlignment::from(v)
            }).collect(),
        }
    }
}

impl From<Alignment> for PyAlignment {
    fn from(alignment: Alignment) -> Self {
        Self {
            penalty: alignment.penalty,
            length: alignment.length,
            query_position: alignment.position.query,
            target_position: alignment.position.target,
            operations: alignment.operations.into_iter().map(|v| {
                PyAlignmentOperations::from(v)
            }).collect(),
        }
    }
}

impl From<AlignmentOperations> for PyAlignmentOperations {
    fn from(op: AlignmentOperations) -> Self {
        Self {
            operation: PyAlignmentOperation::from(op.operation),
            count: op.count,
        }
    }
}

impl From<AlignmentOperation> for PyAlignmentOperation {
    fn from(op: AlignmentOperation) -> Self {
        match op {
            AlignmentOperation::Match => PyAlignmentOperation::Match,
            AlignmentOperation::Subst => PyAlignmentOperation::Subst,
            AlignmentOperation::Insertion => PyAlignmentOperation::Insertion,
            AlignmentOperation::Deletion => PyAlignmentOperation::Deletion,
        }
    }
}
