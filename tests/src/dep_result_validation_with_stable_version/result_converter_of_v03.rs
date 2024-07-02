use sigalign_stable::results::{
    fasta::{
        ReadAlignmentResult as StableReadAlignmentResult,
        FastaAlignmentResult as StableFastaAlignmentResult,
    },
    AlignmentResult as StableAlignmentResult,
    AnchorAlignmentResult as StableAnchorAlignmentResult,
    AlignmentOperations as StableAlignmentOperations,
    AlignmentOperation as StableAlignmentOperation,
};
use sigalign::results::{
    LabeledQueryAlignment,
    LabeledTargetAlignment,
    Alignment,
    AlignmentPosition,
    AlignmentOperations,
    AlignmentOperation,
};


/* Convert stable version to current version */
pub fn convert_result_of_stable_version_to_current(
    stable_result: &StableFastaAlignmentResult
) -> Vec<LabeledQueryAlignment> {
    todo!()
}
fn aln_res_to_aln_res(stable_res: &StableAlignmentResult) -> LabeledQueryAlignment {
    let inner = stable_res.0.iter().map(|rec_aln_res| {
        LabeledTargetAlignment {
            index: rec_aln_res.index as u32,
            label: rec_aln_res.label.clone(),
            alignments: anc_res_to_anc_res(&rec_aln_res.alignments)
        }
    }).collect();
    LabeledQueryAlignment(inner)
}
fn anc_res_to_anc_res(stable_res: &Vec<StableAnchorAlignmentResult>) -> Vec<Alignment> {
    stable_res.iter().map(|anc_res: &StableAnchorAlignmentResult| {
        Alignment {
            penalty: anc_res.penalty as u32,
            length: anc_res.length as u32,
            position: AlignmentPosition {
                target: (anc_res.position.target.0 as u32, anc_res.position.target.1 as u32),
                query: (anc_res.position.query.0 as u32, anc_res.position.query.1 as u32),
            },
            operations: ops_to_ops(&anc_res.operations),
        }
    }).collect()
}
fn ops_to_ops(stable_ops: &Vec<StableAlignmentOperations>) -> Vec<AlignmentOperations> {
    stable_ops.iter().map(|x| {
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