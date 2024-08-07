use sigalign::results::{
    Alignment,
    AlignmentOperations,
    AlignmentOperation,
};

mod generate_and_fill;
mod backtrace;
use backtrace::{
    parse_valid_semi_global_result_from_dpm,
    parse_valid_local_result_from_dpm,
};
mod alignment;
// API for alignment with DP matrix
pub use alignment::{
    dp_semi_global_to_pattern_existing_targets,
    dp_semi_global_to_ref_file,
    dp_semi_global_to_target,
    dp_local_with_one_mat_to_pattern_existing_targets,
    dp_local_with_one_mat_to_ref_file,
    dp_local_with_one_mat_to_target,
    dp_local_with_all_subs_to_pattern_existing_targets,
    dp_local_with_all_subs_to_ref_file,
    dp_local_with_all_subs_to_target,
};

#[derive(Debug, Clone)]
pub struct DpMatrix {
    target: Vec<u8>,
    query: Vec<u8>,
    dp_mat: Vec<Vec<Cell>>,
    ins_mat: Vec<Vec<Cell>>,
    del_mat: Vec<Vec<Cell>>,
}
#[derive(Debug, Clone)]
struct Cell {
    penalty: u32,
    btm: BacktraceMarker,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum BacktraceMarker {
    FromDiag,
    FromIns,
    FromDel,
}

fn dp_matrix_calculates_penalty_accurately() {
    use crate::common::{
        init_logger,
        test_data::DataForValidation,
    };
    use sigalign_utils::sequence_reader::{
        SeqRecord as _,
        fasta::FastaReader,
    };
    use log::info;

    init_logger();

    let qry_count = 10;

    let (ref_file, qry_file) = DataForValidation::Default.get_data_paths();

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
        let results = dp_semi_global_to_ref_file(
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
