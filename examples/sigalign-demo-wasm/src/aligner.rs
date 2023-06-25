use wasm_bindgen::prelude::*;

use super::{
    AlignmentResult,
    Reference,
};
use sigalign::{
    wrapper::{
        DefaultAligner,
    },
};

// Aligner
#[wasm_bindgen]
pub struct Aligner {
    px: u32,
    po: u32,
    pe: u32,
    ml: u32,
    mpl: f32,
    is_local: bool,
    pattern_size: u32,
    inner: DefaultAligner,
}

#[wasm_bindgen]
impl Aligner {
    #[wasm_bindgen(constructor)]
    pub fn new(
        mismatch_penalty: u32,
        gap_open_penalty: u32,
        gap_extend_penalty: u32,
        minimum_aligned_length: u32,
        maximum_penalty_per_length: f32,
        is_local: Option<bool>,
    ) -> Result<Aligner, JsError> {
        let is_local = is_local.unwrap_or(true);
        let inner = if is_local {
            DefaultAligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        } else {
            DefaultAligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        };

        match inner {
            Ok(inner) => {
                let pattern_size = inner.get_pattern_size();
                Ok(Self {
                    px: mismatch_penalty,
                    po: gap_open_penalty,
                    pe: gap_extend_penalty,
                    ml: minimum_aligned_length,
                    mpl: maximum_penalty_per_length,
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
        let alignment_result = self.inner.align_query_labeled(
            reference.as_ref(),
            query.as_bytes(),
        );
        Ok(AlignmentResult::new(alignment_result))
    }
    pub fn drop(self) {
        drop(self)
    }
    #[wasm_bindgen(getter)]
    pub fn px(&self) -> u32 {
        self.px
    }
    #[wasm_bindgen(getter)]
    pub fn po(&self) -> u32 {
        self.po
    }
    #[wasm_bindgen(getter)]
    pub fn pe(&self) -> u32 {
        self.pe
    }
    #[wasm_bindgen(getter)]
    pub fn ml(&self) -> u32 {
        self.ml
    }
    #[wasm_bindgen(getter)]
    pub fn mpl(&self) -> f32 {
        self.mpl
    }
    #[wasm_bindgen(getter)]
    pub fn is_local(&self) -> bool {
        self.is_local
    }
    #[wasm_bindgen(getter)]
    pub fn pattern_size(&self) -> u32 {
        self.pattern_size
    }
    pub fn get_status(&self) -> AlignerStatus {
        AlignerStatus {
            px: self.px,
            po: self.po,
            pe: self.pe,
            ml: self.ml,
            mpl: self.mpl,
            is_local: self.is_local,
        }
    }
}

#[wasm_bindgen]
pub struct AlignerStatus {
    pub px: u32,
    pub po: u32,
    pub pe: u32,
    pub ml: u32,
    pub mpl: f32,
    pub is_local: bool,
}
