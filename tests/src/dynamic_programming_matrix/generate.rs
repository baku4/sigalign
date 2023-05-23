use super::{
    DpMatrix, Cell, BacktraceMarker,
};
use std::cmp;
impl Cell {
    fn new() -> Self {
        Self { penalty: 0, btm: BacktraceMarker::FromDiag }
    }
}
impl DpMatrix {
    pub fn new(
        query: Vec<u8>,
        target: Vec<u8>,
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
    ) -> Self {
        let len1 = query.len();
        let len2 = target.len();
        
        // Initialize
        let mut dp_mat = vec![vec![Cell::new(); len2+1]; len1+1];
        let mut del_mat = vec![vec![Cell::new(); len2+1]; len1+1];
        let mut ins_mat = vec![vec![Cell::new(); len2+1]; len1+1];
        for i in 1..=len1 {
            ins_mat[i][0].penalty = gap_open_penalty;
            del_mat[i][0].penalty = u32::MAX >> 1;
        }
        for j in 1..=len2 {
            del_mat[0][j].penalty = gap_open_penalty;
            ins_mat[0][j].penalty = u32::MAX >> 1;
        }
        
        // Fill matrices
        for i in 1..=len1 {
            for j in 1..=len2 {
                // Deletion
                let p_from_del = {
                    let p_from_dp = dp_mat[i-1][j].penalty + gap_open_penalty + gap_extend_penalty;
                    let p_from_del = del_mat[i-1][j].penalty + gap_extend_penalty;
                    
                    if p_from_dp < p_from_del {
                        del_mat[i][j] = Cell { penalty: p_from_dp, btm: BacktraceMarker::FromDiag };
                        p_from_dp
                    } else {
                        del_mat[i][j] = Cell { penalty: p_from_del, btm: BacktraceMarker::FromDel };
                        p_from_del
                    }
                };
                // Insertion
                let p_from_ins = {
                    let p_from_dp = dp_mat[i][j-1].penalty + gap_open_penalty + gap_extend_penalty;
                    let p_from_ins = ins_mat[i][j-1].penalty + gap_extend_penalty;
                    
                    if p_from_dp < p_from_ins {
                        ins_mat[i][j] = Cell { penalty: p_from_dp, btm: BacktraceMarker::FromDiag };
                        p_from_dp
                    } else {
                        ins_mat[i][j] = Cell { penalty: p_from_ins, btm: BacktraceMarker::FromIns };
                        p_from_ins
                    }
                };
                // DP
                let p_from_diag = {
                    let p_from_dp = dp_mat[i-1][j-1].penalty;
                    p_from_dp + if query[i-1] == target[j-1] {
                        0
                    } else {
                        mismatch_penalty
                    }
                };
                let min_p = p_from_diag.min(p_from_ins.min(p_from_del));
                let btm = if min_p == p_from_del {
                    BacktraceMarker::FromDel
                } else if min_p == p_from_ins {
                    BacktraceMarker::FromIns
                } else {
                    BacktraceMarker::FromDiag
                };
                dp_mat[i][j] = Cell { penalty: min_p, btm };
            }
        }

        Self {
            target,
            query,
            dp_mat,
            del_mat,
            ins_mat,
        }
    }
}

// impl DpMatrix {
//     pub fn new(
//         query: Vec<u8>,
//         target: Vec<u8>,
//         mismatch_penalty: u32,
//         gap_open_penalty: u32,
//         gap_extend_penalty: u32,
//     ) -> Self {
//         let m = query.len() + 1;
//         let n = target.len() + 1;
    
//         let mut dp = vec![
//             vec![(
//                 0,
//                 u32::MAX >> 1,
//                 u32::MAX >> 1,
//             ); n]; m
//         ];
//         for i in 0..m {
//             dp[i][0].1 = gap_open_penalty+gap_extend_penalty;
//         }
//         for j in 0..n {
//             dp[0][j].2 = gap_open_penalty+gap_extend_penalty;
//         }

//         for i in 1..m {
//             for j in 1..n {
//                 let diag = if query[i - 1] == target[j - 1] {
//                     0
//                 } else {
//                     mismatch_penalty
//                 };
    
//                 let m_score = dp[i - 1][j - 1].0 + diag;
//                 let x_score = dp[i - 1][j - 1].1 + diag;
//                 let y_score = dp[i - 1][j - 1].2 + diag;
    
//                 // Match
//                 dp[i][j].0 = cmp::min(
//                     cmp::min(m_score, x_score),
//                     y_score,
//                 );
//                 // Deletion
//                 dp[i][j].1 = cmp::min(
//                     cmp::min(
//                         dp[i - 1][j].0 + gap_open_penalty + gap_extend_penalty,
//                         dp[i - 1][j].1 + gap_extend_penalty,
//                     ),
//                     dp[i - 1][j].2 + gap_open_penalty + gap_extend_penalty,
//                 );
//                 // Insertion
//                 dp[i][j].2 = cmp::min(
//                     cmp::min(
//                         dp[i][j - 1].0 + gap_open_penalty + gap_extend_penalty,
//                         dp[i][j - 1].1 + gap_open_penalty + gap_extend_penalty,
//                     ),
//                     dp[i][j - 1].2 + gap_extend_penalty,
//                 );
//             }
//         }

//         Self {
//             query,
//             target,
//             mismatch_penalty,
//             gap_open_penalty,
//             gap_extend_penalty,
//             matrix: dp,
//         }
//     }
// }