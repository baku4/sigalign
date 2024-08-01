use std::fs::File;
use std::io::{BufRead, BufReader, Write};

use anyhow::Ok;
use sigalign::results::{AlignmentOperation, LabeledQueryAlignment};

use crate::reference::ReferencePathDetector;
use crate::Result;

pub fn write_sam_header<W: Write>(
    mut writer: W,
    reference_path_detector: &ReferencePathDetector,
) -> Result<()> {
    writer.write_all(b"@HD\tVN:1.6\tSO:unsorted\n")?;

    // Fields of manifest file:
    // file_index, file_name, record_index, reference_index, target_index, target_label, target_length
    let manifest_file = File::open(reference_path_detector.get_manifest_file_path())?;
    let reader = BufReader::new(manifest_file);
    for line in reader.lines() {
        let line = line?;
        let fields: Vec<&str> = line.split('\t').collect();

        writer.write_all(format!("@SQ\tSN:{}\tLN:{}\n", fields[5], fields[6]).as_bytes())?;
    }

    Ok(())
}

#[inline]
pub fn extend_sam_line_with_itoa_buffer(
    sam_line_buffer: &mut Vec<u8>,
    query_name: &str,
    is_forward: bool,
    query_length: u32,
    labeled_query_alignment: &LabeledQueryAlignment,
    itoa_buffer: &mut itoa::Buffer,
) {
    let flag = if is_forward { 0 } else { 16 };

    labeled_query_alignment
        .0
        .iter()
        .for_each(|labeled_target_alignment| {
            labeled_target_alignment
                .alignments
                .iter()
                .for_each(|alignment| {
                    sam_line_buffer.extend(query_name.as_bytes());
                    sam_line_buffer.push(b'\t');
                    sam_line_buffer.extend(itoa_buffer.format(flag).as_bytes());
                    sam_line_buffer.push(b'\t');
                    sam_line_buffer.extend(labeled_target_alignment.label.as_bytes());
                    sam_line_buffer.push(b'\t');
                    sam_line_buffer.extend(
                        itoa_buffer
                            .format(alignment.position.target.0 + 1)
                            .as_bytes(),
                    );
                    sam_line_buffer.extend(b"\t255\t");
                    let lclip_size = alignment.position.query.0;
                    if lclip_size != 0 {
                        sam_line_buffer.extend(itoa_buffer.format(lclip_size).as_bytes());
                        sam_line_buffer.push(b'H');
                    }
                    alignment
                        .operations
                        .iter()
                        .for_each(|alignment_operations| {
                            sam_line_buffer
                                .extend(itoa_buffer.format(alignment_operations.count).as_bytes());
                            sam_line_buffer.push(to_cigar_code(&alignment_operations.operation));
                        });
                    let rclip_size = query_length - alignment.position.query.1;
                    if rclip_size != 0 {
                        sam_line_buffer.extend(itoa_buffer.format(rclip_size).as_bytes());
                        sam_line_buffer.push(b'H');
                    }
                    sam_line_buffer.extend(b"\t=\t0\t0\t*\t*\n");
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
