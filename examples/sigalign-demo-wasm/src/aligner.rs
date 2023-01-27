use wasm_bindgen::prelude::*;

use super::{
    SigAligner,
    Reference,
    AlignmentResult,
};

// Aligner
#[wasm_bindgen]
pub struct Aligner {
    px: usize,
    po: usize,
    pe: usize,
    ml: usize,
    mppl: f32,
    is_local: bool,
    pattern_size: usize,
    inner: SigAligner,
}

#[wasm_bindgen]
impl Aligner {
    #[wasm_bindgen(constructor)]
    pub fn new(
        is_local: bool,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> Result<Aligner, JsError> {
        let inner = if is_local {
            SigAligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        } else {
            SigAligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        };

        match inner {
            Ok(inner) => {
                let pattern_size = inner.get_pattern_size();
                Ok(Self {
                    px: mismatch_penalty,
                    po: gap_open_penalty,
                    pe: gap_extend_penalty,
                    ml: minimum_aligned_length,
                    mppl: maximum_penalty_per_length,
                    is_local,
                    pattern_size,
                    inner,
                })
            },
            Err(err) => {
                Err(JsError::new(&format!("{}", err)))
            },
        }
    }
    pub async fn alignment(
        &mut self,
        query: &str,
        reference: &Reference,
    ) -> Result<AlignmentResult, JsError> {
        let alignment_result = self.inner.query_labeled_alignment(
            reference.as_ref(),
            query.as_bytes(),
        );
        match alignment_result {
            Ok(inner_result) => {
                Ok(AlignmentResult::new(inner_result))
            },
            Err(err) => {
                Err(JsError::new(&format!("{}", err)))
            },
        }
    }
    pub fn drop(self) {
        drop(self)
    }
    #[wasm_bindgen(getter)]
    pub fn px(&self) -> usize {
        self.px
    }
    #[wasm_bindgen(getter)]
    pub fn po(&self) -> usize {
        self.po
    }
    #[wasm_bindgen(getter)]
    pub fn pe(&self) -> usize {
        self.pe
    }
    #[wasm_bindgen(getter)]
    pub fn ml(&self) -> usize {
        self.ml
    }
    #[wasm_bindgen(getter)]
    pub fn mppl(&self) -> f32 {
        self.mppl
    }
    #[wasm_bindgen(getter)]
    pub fn is_local(&self) -> bool {
        self.is_local
    }
    #[wasm_bindgen(getter)]
    pub fn pattern_size(&self) -> usize {
        self.pattern_size
    }
    pub fn get_status(&self) -> AlignerStatus {
        AlignerStatus {
            px: self.px,
            po: self.po,
            pe: self.pe,
            ml: self.ml,
            mppl: self.mppl,
            is_local: self.is_local,
            pattern_size: self.pattern_size,
        }
    }
}

#[wasm_bindgen]
pub struct AlignerStatus {
    pub px: usize,
    pub po: usize,
    pub pe: usize,
    pub ml: usize,
    pub mppl: f32,
    pub is_local: bool,
    pub pattern_size: usize,
}
