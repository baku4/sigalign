use sigalign::results::{
    Alignment,
    AlignmentOperations,
    AlignmentOperation,
};

mod generate_and_fill;
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
fn dp_matrix_calculates_penalty_accurately() {
    use crate::common::{
        init_logger,
        test_data_path::{get_qry_for_val_path, get_ref_for_val_path},
    };
    use sigalign_utils::sequence_reader::{
        SeqRecord as _,
        fasta::FastaReader,
    };
    use log::info;

    init_logger();

    let qry_count = 10;

    let qry_file = get_qry_for_val_path();
    let ref_file = get_ref_for_val_path();

    let px = 4;
    let po = 6;
    let pe = 2;
    let minl = 100;
    let maxp = 0.1;

    let mut qry_index = 0;
    let mut qry_buffer = Vec::new();
    let mut qry_reader = FastaReader::from_path(&qry_file).unwrap();
    while let Some(mut record) = qry_reader.next() {
        qry_buffer.clear();
        record.extend_seq_buf(&mut qry_buffer);
        if qry_index == qry_count {
            break;
        }
        info!("Query index: {}", qry_index);
        let results = semi_global_with_dpm(
            &qry_buffer,
            &ref_file,
            px,
            po,
            pe,
            minl,
            maxp,
        );

        results.0.iter().for_each(|x| {
            x.alignments.iter().for_each(|y| {
                let p1 = cal_penalty(&y.operations, px, po, pe);
                let p2 = y.penalty;

                if p1 != p2 {
                    println!("query: {}", String::from_utf8(qry_buffer.clone()).unwrap());
                    println!("Alignment:\n{:#?}", y);
                    panic!("")
                }
            });
        });

        qry_index += 1;
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
