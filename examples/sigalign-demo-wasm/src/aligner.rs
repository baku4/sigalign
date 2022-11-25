use crate::{Result, error_msg};
use super::{SigAligner, SigReference, ReferenceDep};
use super::{Deserialize, Serialize, serialize_to_string};

// Aligner
pub struct AlignerDep {
    pub sig_aligner: Option<SigAligner>,
    state: AlignerState,
}
#[derive(Serialize)]
struct AlignerState {
    exist: bool,
    for_local: bool,
    mismatch_penalty: usize,
    gap_open_penalty: usize,
    gap_extend_penalty: usize,
    minimum_aligned_length: usize,
    maximum_penalty_per_length: f32,
    pattern_size: usize,
}
impl Default for AlignerDep {
    fn default() -> Self {
        Self {
            sig_aligner: None,
            state: AlignerState::default(),
        }
    }
}
impl Default for AlignerState {
    fn default() -> Self {
        Self {
            exist: false,
            for_local: false,
            mismatch_penalty: 0,
            gap_open_penalty: 0,
            gap_extend_penalty: 0,
            minimum_aligned_length: 0,
            maximum_penalty_per_length: 0.0,
            pattern_size: 0,
        }   
    }
}

impl AlignerDep {
    pub fn generate(
        &mut self,
        for_local: bool,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<()> {
        let sig_aligner = if for_local {
            SigAligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?
        } else {
            SigAligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)?
        };
        
        let state = {
            let penalties = sig_aligner.get_penalties();
            let cutoff = sig_aligner.get_similarity_cutoff();
            let pattern_size = sig_aligner.get_pattern_size();

            AlignerState {
                exist: true,
                for_local,
                mismatch_penalty: penalties[0],
                gap_open_penalty: penalties[1],
                gap_extend_penalty: penalties[2],
                minimum_aligned_length: cutoff.0,
                maximum_penalty_per_length: cutoff.1,
                pattern_size,
            }
        };

        self.sig_aligner = Some(sig_aligner);
        self.state = state;

        Ok(())
    }
    pub fn reset(&mut self) {
        *self = Self::default();
    }
    pub fn state_to_json(&self) -> String {
        match serialize_to_string(&self.state) {
            Ok(json) => json,
            Err(error) => format!("{{\"error\": {}}}", error),
        }
    }
}