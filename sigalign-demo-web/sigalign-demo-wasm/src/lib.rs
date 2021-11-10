use wasm_bindgen::prelude::*;
use anyhow::{Result, bail as error_msg};
use serde::{Deserialize, Serialize};
use serde_json::to_string as serialize_to_string;

use sigalign::{Reference as SigReference, Aligner as SigAligner};
use sigalign::reference::LtFmIndexConfig;
use sigalign::reference::basic_sequence_provider::InMemoryProvider;

mod reference;
mod aligner;

use reference::Reference;
use aligner::Aligner;

#[wasm_bindgen]
pub struct Sigalign {
    reference: Reference,
    aligner: Aligner,
}

#[wasm_bindgen]
impl Sigalign {
    pub fn new() -> Self {
        Self {
            reference: Reference::default(),
            aligner: Aligner::default(),
        }
    }
    pub fn generate_reference(
        &mut self,
        fasta: &str,
        is_path: bool,
        sampling_ratio: u64,
        bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) -> String { // If no error occur, return empty String
        let result = if is_path {
            self.reference.generate_with_fasta_file(
                fasta,
                sampling_ratio,
                bwt_block,
                lookup_table_kmer_size,
            )
        } else {
            self.reference.generate_with_fasta_bytes(
                fasta,
                sampling_ratio,
                bwt_block,
                lookup_table_kmer_size,
            )
        };

        match result {
            Ok(_) => String::new(),
            Err(error) => format!("{}", error),
        }        
    }
    pub fn reset_reference(&mut self) {
        self.reference.reset();
    }
    pub fn print_reference_state(&self) -> String {
        self.reference.state_to_json()
    }
    pub fn print_aligner_state(&self) -> String {
        self.aligner.state_to_json()
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
    pub fn reset_aligner(
        &mut self
    ) {
        self.aligner.reset();
    }
}