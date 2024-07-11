// Just to print errored results for each version
use sigalign::{algorithms::{Algorithm, Local, SemiGlobal}, Aligner as CurrentAligner, ReferenceBuilder};
use sigalign_stable::wrapper::{DefaultAligner as StableAligner, DefaultReference as StableReference};
use crate::common::dynamic_programming_matrix::{
    dp_semi_global_to_target,
    dp_local_to_target,
};
use super::stable_result_to_current_result;

const QRY: &[u8; 200] = b"";
const TGT: &[u8; 1000] = b"";
const ALIGNER_OPTION: (u32, u32, u32, u32, f32) = (1, 1, 1, 100, 0.1);
const IS_LOCAL: bool = true;

#[test]
fn print_errored_result_with_current() {
    
    let reference = ReferenceBuilder::new()
        .add_target("target", TGT)
        .build()
        .unwrap();
    
    let result = if IS_LOCAL {
        let mut aligner = CurrentAligner::new(
            Local::new(
                ALIGNER_OPTION.0,
                ALIGNER_OPTION.1,
                ALIGNER_OPTION.2,
                ALIGNER_OPTION.3,
                ALIGNER_OPTION.4
            ).unwrap()
        );
        println!("pattern_size: {}", aligner.get_pattern_size());
        aligner.align(QRY, &reference)
    } else {
        let mut aligner = CurrentAligner::new(
            SemiGlobal::new(
                ALIGNER_OPTION.0,
                ALIGNER_OPTION.1,
                ALIGNER_OPTION.2,
                ALIGNER_OPTION.3,
                ALIGNER_OPTION.4
            ).unwrap()
        );
        println!("pattern_size: {}", aligner.get_pattern_size());
        aligner.align(QRY, &reference)
    };

    for aln in result.0.iter() {
        for aln in aln.alignments.iter() {
            println!("{:?}", aln);
        }
    }
}

#[test]
fn print_errored_result_with_stable() {
    let reference = StableReference::from_fasta_bytes(
        format!(">target\n{}", String::from_utf8_lossy(TGT)).as_bytes()
    ).unwrap();

    let mut aligner = if IS_LOCAL {
        StableAligner::new_local(
            ALIGNER_OPTION.0,
            ALIGNER_OPTION.1,
            ALIGNER_OPTION.2,
            ALIGNER_OPTION.3,
            ALIGNER_OPTION.4
        ).unwrap()
    } else {
        StableAligner::new_semi_global(
            ALIGNER_OPTION.0,
            ALIGNER_OPTION.1,
            ALIGNER_OPTION.2,
            ALIGNER_OPTION.3,
            ALIGNER_OPTION.4
        ).unwrap()
    };
    println!("pattern_size: {}", aligner.get_pattern_size());

    let result = aligner.align_query(&reference, QRY);
    let result = stable_result_to_current_result(result);
    for aln in result.0.iter() {
        for aln in aln.alignments.iter() {
            println!("{:?}", aln);
        }
    }
}

#[test]
fn print_errored_result_with_dp() {
    let result = if IS_LOCAL {
        dp_local_to_target(
            QRY,
            TGT,
            ALIGNER_OPTION.0, ALIGNER_OPTION.1, ALIGNER_OPTION.2, ALIGNER_OPTION.3, ALIGNER_OPTION.4
        )
    } else {
        dp_semi_global_to_target(
            QRY,
            TGT,
            ALIGNER_OPTION.0, ALIGNER_OPTION.1, ALIGNER_OPTION.2, ALIGNER_OPTION.3, ALIGNER_OPTION.4
        )
    };

    for aln in result.iter() {
        println!("{:?}", aln);
    }
}
