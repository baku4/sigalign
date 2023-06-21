use sigalign::{
    reference::{
        sequence_storage::SequenceBuffer,
        extensions::EstimateSize,
    },
    wrapper::{
        DefaultReference,
    },
};
use wasm_bindgen::prelude::*;
use crate::utils::err_to_js_err;
use ahash::AHashSet;

#[wasm_bindgen]
pub struct Reference {
    inner: DefaultReference,
    sorted_characters_in_targets: Vec<u8>,
}

#[wasm_bindgen]
impl Reference {
    pub async fn build(
        fasta: &str,
    ) -> Result<Reference, JsError> {
        let inner = DefaultReference::from_fasta_bytes(
            fasta.as_bytes()
        );
        match inner {
            Ok(inner) => {
                let characters_in_targets = Self::get_sorted_characters_in_targets_from_reference(&inner);
                Ok(Self::new(inner, characters_in_targets))
            },
            Err(err) => {
                Err(err_to_js_err(err.into()))
            },
        }
    }
    fn get_sorted_characters_in_targets_from_reference(inner_reference: &DefaultReference) -> Vec<u8> {
        let mut set = AHashSet::new();
        let mut sequence_buffer = inner_reference.get_sequence_buffer();
        for target_index in 0..inner_reference.num_targets() {
            inner_reference.fill_sequence_buffer(target_index, &mut sequence_buffer);
            set.extend(sequence_buffer.buffered_sequence());
        }
        let mut vec: Vec<u8> = set.into_iter().collect();
        vec.sort_unstable();
        vec
    }
    fn new(
        inner: DefaultReference,
        sorted_characters_in_targets: Vec<u8>,
    ) -> Self {
        Self {
            inner,
            sorted_characters_in_targets,
        }
    }
    pub(crate) fn get_sorted_characters_in_targets(&self) -> Vec<u8> {
        self.sorted_characters_in_targets.clone()
    }
    pub fn drop(self) {
        drop(self)
    }
    pub fn target_count(&self) -> u32 {
        self.inner.num_targets()
    }
    pub fn get_target_sequence(&self, target_index: u32) -> Vec<u8> {
        let mut buffer = self.inner.get_sequence_buffer();
        self.inner.fill_sequence_buffer(target_index, &mut buffer);
        let sequence = buffer.buffered_sequence();
        sequence.to_vec()
    } 
    pub fn get_status(&self) -> ReferenceStatus {
        let target_count = self.target_count();
        let est_byte_size = self.inner.serialized_size();
        ReferenceStatus {
            target_count,
            est_byte_size,
        }
    }
}

impl AsRef<DefaultReference> for Reference {
    fn as_ref(&self) -> &DefaultReference {
        &self.inner
    }
}

#[wasm_bindgen]
pub struct ReferenceStatus {
    pub target_count: u32,
    pub est_byte_size: usize,
}
