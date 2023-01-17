use wasm_bindgen::prelude::*;
// use anyhow::{Result, bail as error_msg};
// use serde::{Deserialize, Serialize};
// use serde_json::to_string as serialize_to_string;

use sigalign::{
    Reference as SigReference,
    ReferenceBuilder as SigReferenceBuilder,
    Aligner as SigAligner,
    sequence_storage::{
        InMemoryStorage as SigInMemoryStorage,
    },
    result::{
        AlignmentLabeledResult,
        FastaAlignmentLabeledResult,
    }
};

mod reference;
mod aligner;
mod result;
use reference::Reference;
use aligner::Aligner;
use result::Result;

struct SigAlignApp {
    reference: Reference,
    aligner: Aligner,
    result: Result,
}

impl SigAlignApp {
    pub fn new() -> Self {
        Self {
            reference: Reference::default(),
            aligner: Aligner::default(),
            result: Result::default(),
        }
    }
    pub fn build_reference(
        &mut self,
        fasta_formatted_string: &str,
        suffix_array_sampling_ratio: usize,
        use_128_sized_bwt_block: usize,
        lookup_table_kmer_size: usize,
    ) {
        //
    }
    pub fn build_aligner(
        &mut self,
        for_local: bool,
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) {
        //
    }
    pub fn alignment_query(
        &mut self,
        query: &str,
    ) {
        //
    }
    pub fn alignment_fasta(
        &mut self,
        fasta_formatted_string: &str,
    ) {
        //
    }
    pub fn get_result_as_json(
        &self,
    ) {
        //
    }
    pub fn get_result_as_tsv(
        &self,
    ) {
        //
    }
}

// #[wasm_bindgen]
// impl Reference {
//     pub fn new_test() -> Self {
//         let mut ss = InMemoryStorage::new();
//         ss.add_record(b"ACACAGATCGCAAACTCACAATTGTATTTCTTTGCCACCTGGGCATATACTTTTTGCGCCCCCTCATTTA", "record_1");
//         ss.add_record(b"TCTGGGGCCATTGTATTTCTTTGCCAGCTGGGGCATATACTTTTTCCGCCCCCTCATTTACGCTCATCAC", "record_2");

//         let builder = SigReferenceBuilder::new();
//         let inner = builder.build(ss).unwrap();
//         Self { inner }
//     }
// }

// #[wasm_bindgen]
// pub struct Aligner {
//     inner: SigAligner,
// }
// #[wasm_bindgen]
// impl Aligner {
//     pub fn new_test() -> Self {
//         let inner = SigAligner::new_local(
//         4,
//         6,
//         2,
//         50,
//         0.2,
//         ).unwrap();
        
//         Self { inner }
//     }
//     pub fn align(&mut self, reference: &Reference, query: &str) -> String {
//         let result = self.inner.query_alignment(&reference.inner, query.as_bytes()).unwrap();
//         result.to_json_pretty()
//     }
// }

// #[test]
// fn test_test() {
//     let reference = Reference::new_test();
//     let mut aligner = Aligner::new_test();

//     let result = aligner.align(&reference, "CAAACTCACAATTGTATTTCTTTGCCAGCTGGGCATATACTTTTTCCGCCCCCTCATTTAACTTCTTGGA");
//     println!("result: {}", result);
// }

// #[wasm_bindgen]
// impl SigAlignApp {
//     #[wasm_bindgen(constructor)]
//     pub fn new() -> Self {
//         Self {
//             reference: Reference::default(),
//             aligner: Aligner::default(),
//         }
//     }
//     #[wasm_bindgen(getter)]
//     pub fn reference(&self) -> String {
//         self.reference.state_to_json()
//     }
//     #[wasm_bindgen(getter)]
//     pub fn aligner(&self) -> String {
//         self.aligner.state_to_json()
//     }
//     pub fn generate_reference(
//         &mut self,
//         fasta_formatted_string: &str,
//         suffix_array_sampling_ratio: usize,
//         use_128_sized_bwt_block: usize,
//         lookup_table_kmer_size: usize,
//     ) -> String {
//         // let result = self.reference.generate_with_fasta_bytes(
//         //     fasta_formatted_string,
//         //     sampling_ratio,
//         //     bwt_block,
//         //     lookup_table_kmer_size,
//         // );

//         // match result {
//         //     Ok(_) => String::new(),
//         //     Err(error) => format!("{}", error),
//         // }
//         "".to_string()
//     }
//     pub fn generate_aligner(
//         &mut self,
//         for_local: bool,
//         mismatch_penalty: usize,
//         gap_open_penalty: usize,
//         gap_extend_penalty: usize,
//         minimum_aligned_length: usize,
//         maximum_penalty_per_length: f32,
//     ) -> String { // If no error occur, return empty String
//         let result = self.aligner.generate(for_local, mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length);

//         match result {
//             Ok(_) => String::new(),
//             Err(error) => format!("{}", error),
//         }
//     }
//     pub fn reset_reference(&mut self) {
//         self.reference.reset()
//     }
//     pub fn reset_aligner(&mut self) {
//         self.aligner.reset()
//     }

//     pub fn alignment(
//         &mut self,
//         query: &str,
//     ) -> String {
//         let result = alignment(&mut self.aligner, &mut self.reference, query);
//         match result {
//             Ok(json) => json,
//             Err(err) => format!("{{\"error\": \"{}\"}}", err),
//         }
//     }
// }

// fn alignment(
//     aligner: &mut Aligner,
//     reference: &mut Reference,
//     query: &str,
// ) -> Result<String> {
//     let sig_aligner = match &mut aligner.sig_aligner {
//         Some(sig_aligner) => sig_aligner,
//         None => {
//             if reference.sig_reference.is_none() {
//                 error_msg!("Reference and Aligner are not generated")
//             } else {
//                 error_msg!("Aligner is not generated")
//             }
//         },
//     };
//     let mut sig_reference = match &mut reference.sig_reference {
//         Some(sig_reference) => sig_reference,
//         None => error_msg!("Reference is not generated")
//     };

//     let result = sig_aligner.query_labeled_alignment(&mut sig_reference, query.as_bytes())?;
//     Ok(result.to_json())
// }
