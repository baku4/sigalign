use sigalign_stable::{
    result::{
        FastaAlignmentResult as StableFastaAlignmentResult,
        ReadAlignmentResult as StableReadAlignmentResult,
        RecordAlignmentResult as StableRecordAlignmentResult,
        AlignmentResult as StableAlignmentResult,
        AnchorAlignmentResult as StableAnchorAlignmentResult,
        AlignmentOperation as StableAlignmentOperations,
        AlignmentCase as StableAlignmentCase,
    },
};
use sigalign::{
    results::{
        fasta::{FastaAlignmentResult, ReadAlignmentResult},
        AlignmentResult,
        TargetAlignmentResult,
        AnchorAlignmentResult,
        AlignmentPosition,
        AlignmentOperations,
        AlignmentOperation,
    }
};

pub fn convert_result_of_stable_version_to_current(
    stable_result: &StableFastaAlignmentResult
) -> FastaAlignmentResult {
    let inner = stable_result.0.iter().map(|read_alignment_result| {
        read_res_to_read_res(read_alignment_result)
    }).collect();
    FastaAlignmentResult(inner)
}
fn read_res_to_read_res(stable_res: &StableReadAlignmentResult) -> ReadAlignmentResult {
    ReadAlignmentResult {
        read: stable_res.read.clone(),
        result: aln_res_to_aln_res(&stable_res.result),
    }
}
fn aln_res_to_aln_res(stable_res: &StableAlignmentResult) -> AlignmentResult {
    let inner = stable_res.0.iter().map(|rec_aln_res| {
        TargetAlignmentResult {
            index: rec_aln_res.index as u32,
            alignments: anc_res_to_anc_res(&rec_aln_res.alignments)
        }
    }).collect();
    AlignmentResult(inner)
}
fn anc_res_to_anc_res(stable_res: &Vec<StableAnchorAlignmentResult>) -> Vec<AnchorAlignmentResult> {
    stable_res.iter().map(|anc_res| {
        AnchorAlignmentResult {
            penalty: anc_res.penalty as u32,
            length: anc_res.length as u32,
            position: AlignmentPosition {
                target: (anc_res.position.record.0 as u32, anc_res.position.record.1 as u32),
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
            operation: match x.case {
                StableAlignmentCase::Match => AlignmentOperation::Match,
                StableAlignmentCase::Subst => AlignmentOperation::Subst,
                StableAlignmentCase::Insertion => AlignmentOperation::Insertion,
                StableAlignmentCase::Deletion => AlignmentOperation::Deletion,
            }
        }
    }).collect()
}