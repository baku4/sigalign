use sigalign::core::ReferenceInterface;
use sigalign::sequence_storage::SequenceBuffer;
use wasm_bindgen::prelude::*;
use crate::utils::err_to_js_err;

use super::{
    SigReference,
    SigReferenceBuilder,
    SigInMemoryStorage,
};

pub(crate) type DefaultReference = SigReference<SigInMemoryStorage>;

#[wasm_bindgen]
pub struct Reference(DefaultReference);

#[wasm_bindgen]
impl Reference {
    pub async fn build(
        klt: Option<usize>,
        sasr: Option<usize>,
        use_128_bwt: Option<bool>,
        fasta: &str,
    ) -> Result<Reference, JsError> {
        // Make Builder
        let builder = Self::get_builder(klt, sasr, use_128_bwt)?;

        // Make SequenceStorage
        let mut ss = SigInMemoryStorage::new();
        ss.add_fasta_bytes(fasta.as_bytes());

        // Build
        match builder.build(ss) {
            Ok(inner) => {
                Ok(Self::new(inner))
            },
            Err(err) => {
                Err(err_to_js_err(err))
            },
        }
    }
    fn get_builder(
        klt: Option<usize>,
        sasr: Option<usize>,
        use_128_bwt: Option<bool>,
    ) -> Result<SigReferenceBuilder, JsError> {
        let mut builder = SigReferenceBuilder::new();
        if let Some(v) = klt {
            builder = match builder.change_count_array_kmer(v) {
                Ok(v) => v,
                Err(err) => return Err(err_to_js_err(err)),
            };
        }
        if let Some(v) = sasr {
            builder = match builder.change_sampling_ratio(v as u64) {
                Ok(v) => v,
                Err(err) => return Err(err_to_js_err(err)),
            };
        }
        if let Some(v) = use_128_bwt {
            if v {
                builder = builder.change_bwt_block_size_to_128();
            } else {
                builder = builder.change_bwt_block_size_to_64();
            }
        }
        Ok(builder)
    }
    fn new(inner_reference: DefaultReference) -> Self {
        Self(inner_reference)
    }
    pub fn drop(self) {
        drop(self)
    }
    pub fn allowed_characters(&self) -> Vec<u8> {
        // self.0.get_allowed_character_list().iter()
        //     .map(|v| v.to_string() )
        //     .collect()
        self.0.get_allowed_character_list().to_vec()
    }
    pub fn record_count(&self) -> usize {
        self.0.total_record_count()
    }
    pub fn get_record_sequence(&self, record_index: usize) -> Vec<u8> {
        let mut buffer = self.0.get_buffer();
        self.0.fill_sequence_buffer(record_index, &mut buffer);
        let sequence = buffer.request_sequence();
        sequence.to_vec()
    } 
    pub fn get_status(&self) -> ReferenceStatus {
        let total_records = self.record_count();
        let is_nucleotide = self.0.get_whether_text_type_is_nucleotide();
        let have_noise = self.0.get_noise_character_of_text_type().is_some();
        let supported_sequences = String::from_utf8(
            self.allowed_characters()
        ).unwrap();
        let klt = self.0.get_lookup_table_kmer_size();
        let sasr = self.0.get_suffix_array_sampling_ratio() as usize;
        let bwt_block_size = self.0.get_size_of_bwt_block();
        let est_byte_size = self.0.size_of();
        ReferenceStatus {
            total_records,
            is_nucleotide,
            have_noise,
            supported_sequences,
            klt,
            sasr,
            bwt_block_size,
            est_byte_size,
        }
    }
}

impl AsRef<DefaultReference> for Reference {
    fn as_ref(&self) -> &DefaultReference {
        &self.0
    }
}

#[wasm_bindgen]
pub struct ReferenceStatus {
    pub total_records: usize,
    // Sequence type
    pub is_nucleotide: bool,
    pub have_noise: bool,
    #[wasm_bindgen(getter_with_clone)]
    pub supported_sequences: String,
    // Compression level
    pub klt: usize,
    pub sasr: usize,
    pub bwt_block_size: usize,
    // Estimated Size
    pub est_byte_size: usize,
}
