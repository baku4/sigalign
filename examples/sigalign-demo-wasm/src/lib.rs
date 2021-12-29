use wasm_bindgen::prelude::*;
use anyhow::{Result, bail as error_msg};
use serde::{Deserialize, Serialize};
use serde_json::to_string as serialize_to_string;

use sigalign::{Reference as SigReference, Aligner as SigAligner};
use sigalign::reference::LtFmIndexConfig;
use sigalign::reference::basic_sequence_provider::InMemoryProvider;

mod reference;
mod aligner;

pub use reference::Reference;
pub use aligner::Aligner;

#[wasm_bindgen]
pub struct Sigalign {
    reference: Reference,
    aligner: Aligner,
}

#[wasm_bindgen]
impl Sigalign {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            reference: Reference::default(),
            aligner: Aligner::default(),
        }
    }
    #[wasm_bindgen(getter)]
    pub fn reference(&self) -> String {
        self.reference.state_to_json()
    }
    #[wasm_bindgen(getter)]
    pub fn aligner(&self) -> String {
        self.aligner.state_to_json()
    }
    pub fn generate_reference(
        &mut self,
        fasta: &str,
        sampling_ratio: usize,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> String {
        let result = self.reference.generate_with_fasta_bytes(
            fasta,
            sampling_ratio,
            bwt_block,
            lookup_table_kmer_size,
        );

        match result {
            Ok(_) => String::new(),
            Err(error) => format!("{}", error),
        }
    }
    pub fn generate_aligner(
        &mut self,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> String { // If no error occur, return empty String
        let result = self.aligner.generate(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length);

        match result {
            Ok(_) => String::new(),
            Err(error) => format!("{}", error),
        }
    }
    pub fn reset_reference(&mut self) {
        self.reference.reset()
    }
    pub fn reset_aligner(&mut self) {
        self.aligner.reset()
    }

    pub fn semi_global_alignment(
        &mut self,
        query: &str,
    ) -> String {
        let result = semi_global_alignment(&mut self.aligner, &mut self.reference, query);
        match result {
            Ok(json) => json,
            Err(err) => format!("{{\"error\": \"{}\"}}", err),
        }
    }
    pub fn local_alignment(
        &mut self,
        query: &str,
    ) -> String {
        let result = local_alignment(&mut self.aligner, &mut self.reference, query);
        match result {
            Ok(json) => json,
            Err(err) => format!("{{\"error\": \"{}\"}}", err),
        }
    }
}

fn semi_global_alignment(
    aligner: &mut Aligner,
    reference: &mut Reference,
    query: &str
) -> Result<String> {
    let sig_aligner = match &mut aligner.sig_aligner {
        Some(sig_aligner) => sig_aligner,
        None => {
            if reference.sig_reference.is_none() {
                error_msg!("Reference and Aligner are not generated")
            } else {
                error_msg!("Aligner is not generated")
            }
        },
    };
    let mut sig_reference = match &mut reference.sig_reference {
        Some(sig_reference) => sig_reference,
        None => error_msg!("Reference is not generated")
    };

    let result = sig_aligner.semi_global_alignment_labeled(&mut sig_reference, query.as_bytes())?;
    Ok(result)
}
fn local_alignment(
    aligner: &mut Aligner,
    reference: &mut Reference,
    query: &str
) -> Result<String> {
    let sig_aligner = match &mut aligner.sig_aligner {
        Some(sig_aligner) => sig_aligner,
        None => {
            if reference.sig_reference.is_none() {
                error_msg!("Reference and Aligner are not generated")
            } else {
                error_msg!("Aligner is not generated")
            }
        },
    };
    let mut sig_reference = match &mut reference.sig_reference {
        Some(sig_reference) => sig_reference,
        None => error_msg!("Reference is not generated")
    };

    let result = sig_aligner.local_alignment_labeled(&mut sig_reference, query.as_bytes())?;
    Ok(result)
}
