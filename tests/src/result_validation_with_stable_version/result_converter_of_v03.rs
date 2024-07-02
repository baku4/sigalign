use sigalign::results::{
    Alignment, AlignmentOperation, AlignmentOperations, AlignmentPosition, QueryAlignment, TargetAlignment
};
use sigalign_stable::results::{
    AlignmentOperation as StableAlignmentOperation,
    AlignmentOperations as StableAlignmentOperations,
    AlignmentResult as StableQueryAlignment,
    AnchorAlignmentResult as StableAnchorAlignmentResult,
};

pub fn stable_result_to_current_result(
    stable_result: StableQueryAlignment,
) -> QueryAlignment {
    let target_alignments = stable_result.0.into_iter().map(|target| {
        TargetAlignment {
            index: target.index,
            alignments: target.alignments.into_iter().map(|alignment| {
                aln_to_aln(alignment)
            }).collect(),
        }
    }).collect();
    QueryAlignment(target_alignments)
}
fn aln_to_aln(stable_aln: StableAnchorAlignmentResult) -> Alignment {
    Alignment {
        penalty: stable_aln.penalty as u32,
        length: stable_aln.length as u32,
        position: AlignmentPosition {
            target: (stable_aln.position.target.0 as u32, stable_aln.position.target.1 as u32),
            query: (stable_aln.position.query.0 as u32, stable_aln.position.query.1 as u32),
        },
        operations: ops_to_ops(stable_aln.operations),
    }
}
fn ops_to_ops(stable_ops: Vec<StableAlignmentOperations>) -> Vec<AlignmentOperations> {
    stable_ops.into_iter().map(|x| {
        AlignmentOperations {
            count: x.count,
            operation: match x.operation {
                StableAlignmentOperation::Match => AlignmentOperation::Match,
                StableAlignmentOperation::Subst => AlignmentOperation::Subst,
                StableAlignmentOperation::Insertion => AlignmentOperation::Deletion,
                StableAlignmentOperation::Deletion => AlignmentOperation::Insertion,
            }
        }
    }).collect()
}