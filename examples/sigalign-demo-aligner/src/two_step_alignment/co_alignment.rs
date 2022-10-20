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

pub fn co_alignment_to_stdout(
    self_desc_reference: SelfDescReference,
    aligner_1: &mut Aligner,
    aligner_2: &mut Aligner,
    fasta_pathbuf: &std::path::PathBuf,
    stdout: &mut std::io::Stdout,
) -> Result<()> {
    let fasta_reader = FastaReader::from_file_path(fasta_pathbuf)?;

    match self_desc_reference {
        SelfDescReference::Ref(inner_ref) => {
            co_alignment_to_writer(
                &inner_ref,
                aligner_1,
                aligner_2,
                fasta_reader,
                stdout,
            )
        },
        SelfDescReference::RecRc(inner_ref) => {
            co_alignment_to_writer(
                &inner_ref,
                aligner_1,
                aligner_2,
                fasta_reader,
                stdout,
            )
        },
    }
}

fn co_alignment_to_writer<R, W, S>(
    reference: &Reference<S>,
    aligner_1: &mut Aligner,
    aligner_2: &mut Aligner,
    mut fasta_reader: FastaReader<R>,
    mut writer: W,
) -> Result<()> where
    R: Read,
    W: Write,
    S: SequenceStorage,
{
    let mut sequence_buffer = reference.get_buffer();

    // First read
    writer.write(b"[")?;
    let mut need_first_written = true;
    while need_first_written {
        if let Some((label, query)) = fasta_reader.next() {
            if reference.searchable(&query) {
                let result_1 = aligner_1.alignment(&reference, &mut sequence_buffer, &query);
                if result_1.result_counts() == 0 {
                    let result_2 = aligner_2.alignment(&reference, &mut sequence_buffer, &query);
                    let read_alignment_result = ReadAlignmentResult {
                        read: label,
                        result: result_2,
                    };
                    read_alignment_result.write_as_json(&mut writer);
                    need_first_written = false;
                } else {
                    let read_alignment_result = ReadAlignmentResult {
                        read: label,
                        result: result_1,
                    };
                    read_alignment_result.write_as_json(&mut writer);
                    need_first_written = false;
                }
            }
        };
    }

    // Middle reads
    fasta_reader.into_iter().for_each(|(label, query)| {
        if reference.searchable(&query) {
            let result_1 = aligner_1.alignment(&reference, &mut sequence_buffer, &query);
            if result_1.result_counts() == 0 {
                let result_2 = aligner_2.alignment(&reference, &mut sequence_buffer, &query);
                let read_alignment_result = ReadAlignmentResult {
                    read: label,
                    result: result_2,
                };
                writer.write(b","); // Do not error check
                read_alignment_result.write_as_json(&mut writer);
            } else {
                let read_alignment_result = ReadAlignmentResult {
                    read: label,
                    result: result_1,
                };
                writer.write(b","); // Do not error check
                read_alignment_result.write_as_json(&mut writer);
            }
        }
        // Ignore unsearchable query
    });

    // Last closing
    writer.write(b"]")?;
    writer.flush()?;

    Ok(())
}