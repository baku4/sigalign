use std::fs;
use std::path::PathBuf;
use std::io::{Read, Write};

use super::{
    Result, error_msg,
    get_ref_for_val_path,
    get_qry_for_val_path,
    get_local_tmp_dir,
};

use sigalign_stable::{
    wrapper::{
        DefaultReference as StableDefaultReference,
        DefaultAligner as StableDefaultAligner,
    },
    results::{
        fasta::FastaAlignmentResult as StableFastaAlignmentResult,
    },
};

use super::ANSWER_ALIGNER_OPTION;
const SEMI_GLOBAL_ANSWER_FILE: &str = "answer_semi_global-v0_3_0-alpha_0.json";
const LOCAL_ANSWER_FILE: &str = "answer_local-v0_3_0-alpha_0.json";

enum Mode {
    SemiGlobal,
    Local,
}
pub fn get_answer_or_generate() -> Result<[StableFastaAlignmentResult; 2]> {
    let semi_global_answer_path = get_semi_global_answer_path()?;
    let local_answer_path = get_local_answer_path()?;

    // Load answer
    let mut optional_stable_reference: Option<_> = None;
    let semi_global_result_answer = get_or_make_answer_result(
        &semi_global_answer_path,
        &mut optional_stable_reference,
        Mode::SemiGlobal,
    )?;
    let local_result_answer = get_or_make_answer_result(
        &local_answer_path,
        &mut optional_stable_reference,
        Mode::Local,
    )?;

    Ok([semi_global_result_answer, local_result_answer])
}
fn get_or_make_answer_result(
    cache_file_path: &PathBuf,
    optional_stable_reference: &mut Option<StableDefaultReference>,
    mode: Mode,
) -> Result<StableFastaAlignmentResult> {
    // Make cache file if not exists
    if !cache_file_path.exists() {
        // Prepare reference
        if optional_stable_reference.is_none() {
            let ref_file = get_ref_for_val_path();
            let reference = StableDefaultReference::from_fasta_file(&ref_file)?;
            *optional_stable_reference = Some(reference);
        };
        let stable_reference = optional_stable_reference.as_ref().unwrap();

        // Prepare Aligner
        let mut stable_aligner = match mode {
            Mode::SemiGlobal => {
                let aligner = StableDefaultAligner::new_semi_global(
                    ANSWER_ALIGNER_OPTION.0,
                    ANSWER_ALIGNER_OPTION.1,
                    ANSWER_ALIGNER_OPTION.2,
                    ANSWER_ALIGNER_OPTION.3,
                    ANSWER_ALIGNER_OPTION.4,
                )?;
                aligner
            },
            Mode::Local => {
                let aligner = StableDefaultAligner::new_local(
                    ANSWER_ALIGNER_OPTION.0,
                    ANSWER_ALIGNER_OPTION.1,
                    ANSWER_ALIGNER_OPTION.2,
                    ANSWER_ALIGNER_OPTION.3,
                    ANSWER_ALIGNER_OPTION.4,
                )?;
                aligner
            },
        };

        // Perform alignment
        let qry_file = get_qry_for_val_path();
        let result = stable_aligner.align_fasta_file(
            stable_reference,
            &qry_file,
        )?;

        // Cache
        save_answer_to_file_as_json(&result, &cache_file_path)?;
    }

    // Load as current SigAlign result
    let result = load_answer_from_json_file(&cache_file_path)?;
    Ok(result)
}

fn get_semi_global_answer_path() -> Result<PathBuf> {
    let mut path = get_local_tmp_dir()?;
    path.push(SEMI_GLOBAL_ANSWER_FILE);
    Ok(path)
}
fn get_local_answer_path() -> Result<PathBuf> {
    let mut path = get_local_tmp_dir()?;
    path.push(LOCAL_ANSWER_FILE);
    Ok(path)
}
fn save_answer_to_file_as_json(fasta_alignment_result: &StableFastaAlignmentResult, path: &PathBuf) -> Result<()> {
    let json_result = fasta_alignment_result.to_json();
    let byte = json_result.as_bytes();

    let mut file = fs::File::create(path)?;
    file.write_all(&byte)?;
    file.flush()?;
    
    Ok(())
}
fn load_answer_from_json_file(json_result: &PathBuf) -> Result<StableFastaAlignmentResult> {
    let mut file = fs::File::open(json_result)?;
    let mut byte = Vec::new();
    file.read_to_end(&mut byte)?;
    let json_string = String::from_utf8(byte)?;
    let result = StableFastaAlignmentResult::from_json(&json_string).unwrap();
    
    Ok(result)
}