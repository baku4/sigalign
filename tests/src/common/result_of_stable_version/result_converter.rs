use ahash::AHashMap;
use sigalign_stable::{
    results::{
        fasta::{
            ReadAlignmentResult as StableReadAlignmentResult,
            FastaAlignmentResult as StableFastaAlignmentResult,
        },
        AlignmentResult as StableAlignmentResult,
        AnchorAlignmentResult as StableAnchorAlignmentResult,
        AlignmentOperations as StableAlignmentOperations,
        AlignmentOperation as StableAlignmentOperation,
    },
};
use sigalign_core::{
    results::{
        QueryAlignment,
        TargetAlignment,
        Alignment,
        AlignmentPosition,
        AlignmentOperations,
        AlignmentOperation,
    }
};

pub fn convert_result_of_stable_version_to_current_core_result(
    stable_result: &StableFastaAlignmentResult
) -> AHashMap<String, QueryAlignment> {
    stable_result.0.iter().map(|read_alignment_result| {
        let read = read_alignment_result.read.clone();
        let result = aln_res_to_aln_res(&read_alignment_result.result);
        (read, result)
    }).collect()
}
fn aln_res_to_aln_res(stable_res: &StableAlignmentResult) -> QueryAlignment {
    let inner = stable_res.0.iter().map(|rec_aln_res| {
        TargetAlignment {
            index: rec_aln_res.index as u32,
            alignments: anc_res_to_anc_res(&rec_aln_res.alignments)
        }
    }).collect();
    QueryAlignment(inner)
}
fn anc_res_to_anc_res(stable_res: &Vec<StableAnchorAlignmentResult>) -> Vec<Alignment> {
    stable_res.iter().map(|anc_res| {
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