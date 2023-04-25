use sigalign::results::{
    AlignmentOperations,
    AlignmentOperation, AnchorAlignmentResult,
};
use std::cmp;

mod generate;
mod backtrace;

pub struct DpMatrix {
    query: Vec<u8>,
    target: Vec<u8>,
    mismatch_penalty: u32,
    gap_open_penalty: u32,
    gap_extend_penalty: u32,
    matrix: Vec<Vec<(u32, u32, u32)>>,
}

#[test]
fn test_dp_matrix() {
    let query = b"CTCTACACTACCTGCCTGGCCAGCAGATCCGCCCTGTCTATACTACCTGCCGCTCCTGCGGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGACCCGCCCTGTCTACACTACCTGCCTGGTCAGTATATCCACCCTATCTACACTACCTGCCTGGCCAGCATATCCGCCCTGTCTACACTACCTGCCAGCCCAGCA";
    let target = b"GCTTTTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGATCAACACTGTCTACACTACCTGCTTTTCCAGCAGATCCACACTGTCTACACTACCTGCTTTTCCAGCAGATCCACCCCGTCTACACTACCTGCCTGGCCAGCATATCCACCCTGTCTACACTACCTGCTTTTCCAGTAGATCTGCCCTATCTACAATACCTGCTTGTCCAGCAGAACCACCCTGTCTATACTACCTGCCTGTCCAGCAGAACCACCCTGTCTATACTACCTGCCTGTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGGCCAGCAGATCCGCCCTGTCTATACTACCTGCCGCTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAGCAGACCCGCCCTGTCTACACTACCTGCCTGGTCAGTATATCCACCCTATCTACACTACCTGCCTGGCCAGCATATCCGCCCTGTCTACACTACCTGCCAGCCCAGCAGATCCGCCCTGTCTACACTACCTGCCTGGCCAGTAGATCCACGCTATCTACACTACCTGCCTGGCCAGCAGATCCACCCTGTCAACACTACCTGCTTGTCCAGCAGGTCCACACTGTCTACACTACCTGCCTGTCCAGCAGGTGCACCCTATCTACACTACCTGACTGTCCAGCAGATCCACCCTGTCTACACTACCTGCCTGTCCAAAAGATCCACCCTGTCTATATTACCTGCCTATACAGCAGAACTACCCTGTCTACACTACCAGCCTCCCCAGCAGATCCACCCTGTCTATACTACCTGCCTGGCCAGTAGATGCATCCTGTCTTCACTACCTGCTTGTCCAGCAGGTCCACCATGTCTACACTGCCTGCCTGGCCAGCAGATCCACCCTGTCTACACTACCTGCCTGCAAAGCAGATCCACCCTGTCTACACTACCTGGCTGGCCAGTAGATCCACGCTATCTACACTACCTTCCTGTCCAGCAGATCCAAC";

    let mismatch_penalty = 4;
    let gap_open_penalty = 6;
    let gap_extend_penalty = 2;
    
    println!("# Start to generate DpMatrix");
    let dp_matrix = DpMatrix::new(
        query.to_vec(),
        target.to_vec(),
        mismatch_penalty,
        gap_open_penalty,
        gap_extend_penalty,
    );
    println!("# Start to parse result");
    let result = dp_matrix.parse_the_valid_semi_global_result(
        50,
        0.1,
    );

    for (idx, v) in result.iter().enumerate() {
        println!("# idx: {:?}", idx);
        println!("{:#?}", v);
    }

    // let (score, aligned_seq1, aligned_seq2) = sequence_alignment(
    //     seq1,
    //     seq2,
    //     gap_open,
    //     gap_extend,
    //     mismatch_penalty,
    // );

    // println!("Score: {}", score);
    // println!("Aligned seq1: {:?}", aligned_seq1);
    // println!("Aligned seq1: {:?}", aligned_seq2);
}