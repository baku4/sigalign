use sigalign::results::{
    AlignmentOperations,
    AlignmentOperation, AnchorAlignmentResult,
};
use std::cmp;

mod generate;
mod backtrace;
mod alignment;
pub use alignment::{
    local_all_substring_with_dpm_only_to_pattern_matched_targets,
    semi_global_with_dpm,
};

#[derive(Debug, Clone)]
pub struct DpMatrix {
    target: Vec<u8>,
    query: Vec<u8>,
    dp_mat: Vec<Vec<Cell>>,
    del_mat: Vec<Vec<Cell>>,
    ins_mat: Vec<Vec<Cell>>,
}
#[derive(Debug, Clone)]
struct Cell {
    penalty: u32,
    btm: BacktraceMarker,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BacktraceMarker {
    FromDiag,
    FromDel,
    FromIns,
}

#[test]
fn calculation_of_penalty_is_accurate() {
    use crate::test_data_path::{get_qry_for_val_path, get_ref_for_val_path};
    use sigalign::utils::FastaReader;
    use crate::init_logger;
    use log::info;

    init_logger();

    let qry_count = 10;

    let qry_file = get_qry_for_val_path();
    let ref_file = get_ref_for_val_path();

    let px = 4;
    let po = 6;
    let pe = 2;
    let ml = 100;
    let mppl = 0.1;

    for (qry_idx, (label, query)) in FastaReader::from_path(&qry_file).unwrap().enumerate() {
        if qry_idx == qry_count {
            break;
        }
        info!("Query label: {}", label);
        let results = semi_global_with_dpm(
            &query,
            &ref_file,
            px,
            po,
            pe,
            ml,
            mppl,
        );

        results.0.iter().for_each(|x| {
            x.alignments.iter().for_each(|y| {
                let p1 = cal_penalty(&y.operations, px, po, pe);
                let p2 = y.penalty;

                if p1 != p2 {
                    println!("query: {}", String::from_utf8(query.clone()).unwrap());
                    println!("AnchorAlignmentResult:\n{:#?}", y);
                    panic!("")
                }
            });
        });
    }
}
fn cal_penalty(
    operations: &Vec<AlignmentOperations>,
    px: u32,
    po: u32,
    pe: u32,
) -> u32 {
    operations.iter().map(|x| {
        match x.operation {
            AlignmentOperation::Match => {
                0
            },
            AlignmentOperation::Subst => {
                px * x.count
            },
            AlignmentOperation::Insertion | AlignmentOperation::Deletion => {
                pe * x.count + po
            },
        }
    }).sum()
}

// #[test]
// fn test_errored_seq() {
//     use backtrace::*;

//     let query = b"GCCTAGACAGAAGAATTCCCAGTAACTTCCTTGTGTTGTGTGCATTCAACTCACAGAGTTGAACGTTCCCTTAGACAGAGCAGATTTGAAACACTCTATTTGTGCAATTTGCAAATGTAGATTTCAAGCGCTTTAAGGTCAATGGCAGAAAAGGAAATATCTTCGTTTCAAAACTAGA";

//     let target = b"TTTCTGTTCATAXXXTAGACAGAAGAATTCCCAGTAACTTCCTTGTGTTGTGTGCATTCAACTCACAGAGTTGAACGTTCCCTTAGACAGAGCAGATTTGAAACACTCTATTTGTGCAATTTGCAAATGTAGATTTCAAGCGCTTTAAGGTCAATGGCAGAAAAGGAAATATCTTCGTTTCAAAACTAGAXXXX";

//     let dp_matrix = DpMatrix::new(
//         query.to_vec(),
//         target.to_vec(),
//         4,
//         6,
//         2,
//     );

//     let result = backtrace_from_the_indices(
//         &dp_matrix,
//         0,
//         177,
//         189,
//     );

//     let ops = concat_ops(result.0);
//     println!("ops: {:#?}", ops);
//     let pos = get_alignment_position(177, 189, &ops);
//     println!("pos: {:#?}", pos);
//     println!("pen: {}", result.2);
// }
