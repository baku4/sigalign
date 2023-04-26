use super::DpMatrix;
use std::cmp;

impl DpMatrix {
    pub fn new(
        query: Vec<u8>,
        target: Vec<u8>,
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
    ) -> Self {
        let m = query.len() + 1;
        let n = target.len() + 1;
    
        let mut dp = vec![
            vec![(
                0,
                gap_open_penalty+gap_extend_penalty,
                gap_open_penalty+gap_extend_penalty,
            ); n]; m
        ];

        for i in 1..m {
            for j in 1..n {
                let diag = if query[i - 1] == target[j - 1] {
                    0
                } else {
                    mismatch_penalty
                };
    
                let m_score = dp[i - 1][j - 1].0 + diag;
                let x_score = dp[i - 1][j - 1].1 + diag;
                let y_score = dp[i - 1][j - 1].2 + diag;
    
                // Match
                dp[i][j].0 = cmp::min(
                    cmp::min(m_score, x_score),
                    y_score,
                );
                // Deletion
                dp[i][j].1 = cmp::min(
                    cmp::min(
                        dp[i - 1][j].0 + gap_open_penalty + gap_extend_penalty,
                        dp[i - 1][j].1 + gap_extend_penalty,
                    ),
                    dp[i - 1][j].2 + gap_open_penalty + gap_extend_penalty,
                );
                // Insertion
                dp[i][j].2 = cmp::min(
                    cmp::min(
                        dp[i][j - 1].0 + gap_open_penalty + gap_extend_penalty,
                        dp[i][j - 1].1 + gap_open_penalty + gap_extend_penalty,
                    ),
                    dp[i][j - 1].2 + gap_extend_penalty,
                );
            }
        }

        Self {
            query,
            target,
            mismatch_penalty,
            gap_open_penalty,
            gap_extend_penalty,
            matrix: dp,
        }
    }
}