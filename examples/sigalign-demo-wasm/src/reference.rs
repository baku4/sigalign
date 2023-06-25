use sigalign::{
    reference::{
        Reference as SigReference,
        sequence_storage::{
            SequenceBuffer,
            in_memory::InMemoryStorage,
        },
        pattern_index::{
            lfi::{DynamicLfi, DynamicLfiOption},
        },
        extensions::EstimateSize,
    },
};
use wasm_bindgen::prelude::*;
use crate::utils::err_to_js_err;
use ahash::AHashSet;

type DefaultReference = SigReference<DynamicLfi, InMemoryStorage>;

#[wasm_bindgen]
pub struct Reference {
    inner: DefaultReference,
    sorted_characters_in_targets: Vec<u8>,
}

#[wasm_bindgen]
impl Reference {
    pub async fn build(
        fasta: &str,
        sasr: Option<usize>, // suffix_array_sampling_ratio
        lts: Option<usize>, // max_lookup_table_size
    ) -> Result<Reference, JsError> {
        let sasr = sasr.unwrap_or(1) as u64;
        let lts = lts.unwrap_or(100_000);
        let pattern_index_option = DynamicLfiOption {
            suffix_array_sampling_ratio: sasr,
            max_lookup_table_byte_size: lts,
        };

        let mut sequence_storage = InMemoryStorage::new();
        sequence_storage.add_fasta_bytes(fasta.as_bytes());
        
        match DefaultReference::new(sequence_storage, pattern_index_option) {
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
    pub fn num_targets(&self) -> u32 {
        self.inner.num_targets()
    }
    pub fn get_target_sequence(&self, target_index: u32) -> Vec<u8> {
        let mut buffer = self.inner.get_sequence_buffer();
        self.inner.fill_sequence_buffer(target_index, &mut buffer);
        let sequence = buffer.buffered_sequence();
        sequence.to_vec()
    }
    pub fn get_status(&self) -> ReferenceStatus {
        let num_targets = self.num_targets();
        let est_byte_size = self.inner.serialized_size();
        ReferenceStatus {
            num_targets,
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
    pub num_targets: u32,
    pub est_byte_size: usize,
}
