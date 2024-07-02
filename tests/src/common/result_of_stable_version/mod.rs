// Test if result of current repository == result of stable version of SigAlign
// Answer is created from the SigAlign of version in `crate`
use std::path::PathBuf;
use std::io::{Read, Write};

use crate::common::{
    Result, error_msg,
    init_logger,
    test_data_path::*,
};

use ahash::AHashMap;
use log::{info, error};
mod result_converter;
use result_converter::convert_result_of_stable_version_to_current_core_result;

use sigalign_stable::{
    wrapper::{
        DefaultReference as StableDefaultReference,
        DefaultAligner as StableDefaultAligner,
    },
    results::fasta::FastaAlignmentResult as StableFastaAlignmentResult,
};
use sigalign_core::results::QueryAlignment;

const VERSION_SIGNATURE: &str = "v0_3_2";
const PRECISION_SCALE: u32 = 100_000;

pub fn get_semi_global_result_of_stable_version(
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    query_fasta_file: &PathBuf,
    reference_fasta_file: &PathBuf,
    tmp_dir: &PathBuf,
) -> AHashMap<String, QueryAlignment> {
    info!("Getting result of stable version of SigAlign...");
    let result_cache_file_path = get_result_cache_file_path(
        "semi_global",
        px,
        po,
        pe,
        minl,
        maxp,
        query_fasta_file,
        reference_fasta_file,
        tmp_dir,
    );
    info!("Result cache file path: {:?}", result_cache_file_path);
    // Save Result
    if !result_cache_file_path.exists() {
        info!("Result cache file not found. Generating result...");
        let mut result_cache_file = std::fs::File::create(&result_cache_file_path).unwrap();
        let reference = get_reference(reference_fasta_file);
        let mut aligner = StableDefaultAligner::new_semi_global(px, po, pe, minl, maxp).unwrap();
        let alignment_result = aligner.align_fasta_file(&reference, query_fasta_file).unwrap();
        let json_result = alignment_result.to_json();
        let byte = json_result.as_bytes();
        result_cache_file.write_all(&byte).unwrap();
        result_cache_file.flush().unwrap();
    }
    // Load Result
    info!("Loading result from cache file...");
    let result = get_result_from_file(&result_cache_file_path);
    convert_result_of_stable_version_to_current_core_result(&result)
}

pub fn get_local_result_of_stable_version(
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    query_fasta_file: &PathBuf,
    reference_fasta_file: &PathBuf,
    tmp_dir: &PathBuf,
) -> AHashMap<String, QueryAlignment> {
    info!("Getting result of stable version of SigAlign...");
    let result_cache_file_path = get_result_cache_file_path(
        "local",
        px,
        po,
        pe,
        minl,
        maxp,
        query_fasta_file,
        reference_fasta_file,
        tmp_dir,
    );
    info!("Result cache file path: {:?}", result_cache_file_path);
    // Save Result
    if !result_cache_file_path.exists() {
        info!("Result cache file not found. Generating result...");
        let mut result_cache_file = std::fs::File::create(&result_cache_file_path).unwrap();
        let reference = get_reference(reference_fasta_file);
        let mut aligner = StableDefaultAligner::new_local(px, po, pe, minl, maxp).unwrap();
        let alignment_result = aligner.align_fasta_file(&reference, query_fasta_file).unwrap();
        let json_result = alignment_result.to_json();
        let byte = json_result.as_bytes();
        result_cache_file.write_all(&byte).unwrap();
        result_cache_file.flush().unwrap();
    }
    // Load Result
    info!("Loading result from cache file...");
    let result = get_result_from_file(&result_cache_file_path);
    convert_result_of_stable_version_to_current_core_result(&result)
}

fn get_result_from_file(result_cache_file_path: &PathBuf) -> StableFastaAlignmentResult {
    let mut result_cache_file = std::fs::File::open(&result_cache_file_path).unwrap();
    let mut byte = Vec::new();
    result_cache_file.read_to_end(&mut byte).unwrap();
    let json_string = String::from_utf8(byte).unwrap();
    let result = StableFastaAlignmentResult::from_json(&json_string).unwrap();
    result
}
fn get_reference(reference_fasta_file: &PathBuf) -> StableDefaultReference {
    StableDefaultReference::from_fasta_file(reference_fasta_file).unwrap()
}
fn get_result_cache_file_path(
    algorithm: &str,
    px: u32,
    po: u32,
    pe: u32,
    minl: u32,
    maxp: f32,
    query_fasta_file: &PathBuf,
    reference_fasta_file: &PathBuf,
    tmp_dir: &PathBuf,
) -> PathBuf {
    let mut cache_file_path = tmp_dir.clone();
    let file_stem = format!(
        "{}-{}-{}-{}-{}-{}-{}-{}-{}",
        query_fasta_file.file_stem().unwrap().to_str().unwrap(),
        reference_fasta_file.file_stem().unwrap().to_str().unwrap(),
        VERSION_SIGNATURE,
        algorithm,
        px,
        po,
        pe,
        minl,
        (maxp * PRECISION_SCALE as f32).round() as u32,
    );
    cache_file_path.push(file_stem);
    cache_file_path
}
