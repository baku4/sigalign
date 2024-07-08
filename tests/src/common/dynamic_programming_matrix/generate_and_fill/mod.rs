use super::{
    DpMatrix, Cell, BacktraceMarker,
};
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
        for i in 0..=len1 {
            ins_mat[i][0].penalty = u32::MAX >> 1;
            del_mat[i][0].penalty = u32::MAX >> 1;
        }
        for j in 0..=len2 {
            del_mat[0][j].penalty = u32::MAX >> 1;
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
                let (p_from_diag, is_match) = {
                    let p_from_dp = dp_mat[i-1][j-1].penalty;
                    if query[i-1] == target[j-1] {
                        (p_from_dp, true)
                    } else {
                        (p_from_dp + mismatch_penalty, false)
                    }
                };
                let min_p = p_from_diag.min(p_from_ins.min(p_from_del));
                let btm = if (min_p == p_from_diag) && is_match {
                    BacktraceMarker::FromDiag
                } else if min_p == p_from_del {
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
