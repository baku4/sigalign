use std::io::{Read, Write};
use super::{Result, format_err, error_msg};

use super::{
    ReferencePaths,
    SelfDescReference,
};
use sigalign::{
    core::ReferenceInterface,
    Reference,
    result::ReadAlignmentResult,
    ReferenceBuilder,
    sequence_storage::{SequenceStorage, InMemoryStorage, InMemoryRcStorage},
    Aligner,
    util::FastaReader,
};

pub fn count_fasta_query(fasta_pathbuf: &std::path::PathBuf) -> usize {
    FastaReader::from_file_path(fasta_pathbuf).unwrap().count()
}

// 1st step

pub fn write_fasta_alignment_to_file_checking_mapped(
    self_desc_reference: SelfDescReference,
    aligner: &mut Aligner,
    fasta_pathbuf: &std::path::PathBuf,
    file: &mut std::fs::File,
    unmapped_sorted_query_idx: &mut Vec<usize>,
) -> Result<()> {
    let fasta_reader = FastaReader::from_file_path(fasta_pathbuf)?;

    match self_desc_reference {
        SelfDescReference::Ref(inner_ref) => {
            write_fasta_alignment_json_from_reader_checking_mapped_query(
                &inner_ref,
                aligner,
                fasta_reader,
                file,
                unmapped_sorted_query_idx,
            )
        },
        SelfDescReference::RecRc(inner_ref) => {
            write_fasta_alignment_json_from_reader_checking_mapped_query(
                &inner_ref,
                aligner,
                fasta_reader,
                file,
                unmapped_sorted_query_idx,
            )
        },
    }
}
fn write_fasta_alignment_json_from_reader_checking_mapped_query<R, W, S>(
    reference: &Reference<S>,
    aligner: &mut Aligner,
    mut fasta_reader: FastaReader<R>,
    mut writer: W,
    unmapped_sorted_query_idx: &mut Vec<usize>,
) -> Result<()> where
    R: Read,
    W: Write,
    S: SequenceStorage,
{
    let mut sequence_buffer = reference.get_buffer();

    let mut qry_idx = 0;
    let mut is_first_result = true;

    // First read
    writer.write(b"[")?;
    fasta_reader.into_iter().for_each(|(label, query)| {
        if reference.searchable(&query) {
            let read_alignment_result = ReadAlignmentResult {
                read: label,
                result: aligner.alignment(reference, &mut sequence_buffer, &query),
            };
            if read_alignment_result.result_counts() != 0 {
                if let Ok(pos) = unmapped_sorted_query_idx.binary_search(&qry_idx) {
                    unmapped_sorted_query_idx.remove(pos);
                }
                if is_first_result {
                    read_alignment_result.write_as_json(&mut writer);
                    is_first_result = false;
                } else {
                    writer.write(b","); // Do not error check
                    read_alignment_result.write_as_json(&mut writer);
                }
            }
        }

        qry_idx += 1;
    });
    writer.write(b"]")?;
    writer.flush()?;

    Ok(())
}

// 2nd step

pub fn write_fasta_alignment_to_file_using_unmapped(
    self_desc_reference: SelfDescReference,
    aligner: &mut Aligner,
    fasta_pathbuf: &std::path::PathBuf,
    file: &mut std::fs::File,
    unmapped_sorted_query_idx: &mut Vec<usize>,
) -> Result<()> {
    let fasta_reader = FastaReader::from_file_path(fasta_pathbuf)?;

    match self_desc_reference {
        SelfDescReference::Ref(inner_ref) => {
            write_fasta_alignment_json_from_reader_using_unmapped_query(
                &inner_ref,
                aligner,
                fasta_reader,
                file,
                unmapped_sorted_query_idx,
            )
        },
        SelfDescReference::RecRc(inner_ref) => {
            write_fasta_alignment_json_from_reader_using_unmapped_query(
                &inner_ref,
                aligner,
                fasta_reader,
                file,
                unmapped_sorted_query_idx,
            )
        },
    }
}
fn write_fasta_alignment_json_from_reader_using_unmapped_query<R, W, S>(
    reference: &Reference<S>,
    aligner: &mut Aligner,
    mut fasta_reader: FastaReader<R>,
    mut writer: W,
    mapped_sorted_query_idx: &mut Vec<usize>,
) -> Result<()> where
    R: Read,
    W: Write,
    S: SequenceStorage,
{
    let mut sequence_buffer = reference.get_buffer();

    let mut next_qry_idx_of_reader = 0;
    let mut idx_of_qry_idx_vec = 0;
    let mut is_first_result = true;

    // First read
    writer.write(b"[")?;
    while idx_of_qry_idx_vec < mapped_sorted_query_idx.len() {
        let next_qry_idx_to_map = mapped_sorted_query_idx[idx_of_qry_idx_vec];

        // Skip mapped qry
        for _ in 0..next_qry_idx_to_map-next_qry_idx_of_reader {
            fasta_reader.next();
        }

        if let Some((label, query)) = fasta_reader.next() {
            if reference.searchable(&query) {
                let read_alignment_result = ReadAlignmentResult {
                    read: label,
                    result: aligner.alignment(reference, &mut sequence_buffer, &query),
                };
                
                if read_alignment_result.result_counts() == 0 { // (1) No result
                    idx_of_qry_idx_vec += 1;
                } else { // (2) Have result - write result
                    if is_first_result {
                        read_alignment_result.write_as_json(&mut writer);
                        is_first_result = false;
                    } else {
                        writer.write(b",");
                        read_alignment_result.write_as_json(&mut writer);
                    }

                    mapped_sorted_query_idx.remove(idx_of_qry_idx_vec);
                }
            }
        }

        next_qry_idx_of_reader = next_qry_idx_to_map + 1;
    }
    writer.write(b"]")?;
    writer.flush()?;

    Ok(())
}