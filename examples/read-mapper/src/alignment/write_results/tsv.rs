use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use anyhow::Result;
use sigalign::results::{AlignmentOperation, AlignmentOperations, LabeledQueryAlignment};

pub fn write_tsv_header<W: Write>(mut writer: W) -> Result<()> {
    writer.write_all(
        b"query_name\tis_forward\ttarget_name\tquery_start\tquery_end\ttarget_start\ttarget_end\toperations\n"
    )?;
    Ok(())
}

pub fn extend_tsv_line_with_itoa_buffer(
    tsv_line_buffer: &mut Vec<u8>,
    query_name: &str,
    is_forward: bool,
    labeled_query_alignment: &LabeledQueryAlignment,
    itoa_buffer: &mut itoa::Buffer,
) {
    labeled_query_alignment
        .0
        .iter()
        .for_each(|labeled_target_alignment| {
            labeled_target_alignment
                .alignments
                .iter()
                .for_each(|alignment| {
                    tsv_line_buffer.extend(query_name.as_bytes());
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer.extend(
                        itoa_buffer
                            .format(if is_forward { 1 } else { 0 })
                            .as_bytes(),
                    );
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer.extend(labeled_target_alignment.label.as_bytes());
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer
                        .extend(itoa_buffer.format(alignment.position.query.0).as_bytes());
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer
                        .extend(itoa_buffer.format(alignment.position.query.1).as_bytes());
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer
                        .extend(itoa_buffer.format(alignment.position.target.0).as_bytes());
                    tsv_line_buffer.push(b'\t');
                    tsv_line_buffer
                        .extend(itoa_buffer.format(alignment.position.target.1).as_bytes());
                    tsv_line_buffer.push(b'\t');
                    alignment
                        .operations
                        .iter()
                        .for_each(|alignment_operations| {
                            tsv_line_buffer
                                .extend(itoa_buffer.format(alignment_operations.count).as_bytes());
                            tsv_line_buffer.push(to_cigar_code(&alignment_operations.operation));
                        });
                    tsv_line_buffer.push(b'\n');
                })
        });
}

#[inline(always)]
fn to_cigar_code(alignment_operation: &AlignmentOperation) -> u8 {
    match alignment_operation {
        AlignmentOperation::Match => b'=',
        AlignmentOperation::Subst => b'X',
        AlignmentOperation::Insertion => b'I',
        AlignmentOperation::Deletion => b'D',
    }
}
