use crate::{error_msg, Result};

use std::path::PathBuf;

use sigalign_utils::file_extension_checker::{is_fasta_file, is_fastq_file, is_gzip_file};

/// Return (is_gzip_compressed, is_fasta_file)
/// if is_fasta_file is false, it is a FASTQ file
pub fn check_input_file_extension_is_allowed(input_file: &PathBuf) -> Result<(bool, bool)> {
    // if input_file.is_dir() {
    //     error_msg!("Input file ({}) must be a file", input_file.display())
    // } else if !input_file.exists() {
    //     error_msg!("Input file ({}) does not exist", input_file.display())
    // } FIXME: Revive later

    let is_gzip_compressed = is_gzip_file(input_file);
    let is_fasta_file = if is_gzip_compressed {
        is_fasta_file(input_file.with_extension(""))
    } else {
        is_fasta_file(input_file)
    };
    let is_fastq_file = if is_gzip_compressed {
        is_fastq_file(input_file.with_extension(""))
    } else {
        is_fastq_file(input_file)
    };

    if !is_fasta_file && !is_fastq_file {
        error_msg!(
            "Input file ({}) is not a FASTA or FASTQ file",
            input_file.display()
        )
    }

    Ok((is_gzip_compressed, is_fasta_file))
}
