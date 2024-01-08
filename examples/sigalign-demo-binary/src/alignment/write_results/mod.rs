use std::io::{BufWriter, StdoutLock, Write as _};

use sigalign_core::results::{AlignmentResult, TargetAlignmentResult, AlignmentOperation, AlignmentOperations};

pub trait Direction {
    const IS_FORWARD : u8;
}
pub struct ForwardDirection;
impl Direction for ForwardDirection { const IS_FORWARD : u8 = 1; }
pub struct ReverseDirection;
impl Direction for ReverseDirection { const IS_FORWARD : u8 = 0; }

#[inline(always)]
pub fn write_alignment_result_as_tsv<D: Direction>(
    result: AlignmentResult,
    buf_writer: &mut BufWriter<StdoutLock>,
    itoa_buffer: &mut itoa::Buffer,
    ref_idx: &usize,
    query_id: &[u8],
) {
    result.0.into_iter().for_each(|TargetAlignmentResult {
        index: target_index,
        alignments: anchor_results,
    }| {
        anchor_results.into_iter().for_each(|anchor_result| {
            let _ = buf_writer.write(query_id).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(D::IS_FORWARD).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(*ref_idx).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(target_index).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.penalty).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.length).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.query.0).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.query.1).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.target.0).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            let _ = buf_writer.write(itoa_buffer.format(anchor_result.position.target.1).as_bytes()).unwrap();
            let _ = buf_writer.write(b"\t").unwrap();
            anchor_result.operations.iter().for_each(|AlignmentOperations {
                operation,
                count,
            }| {
                let _ = buf_writer.write(itoa_buffer.format(*count).as_bytes()).unwrap();
                let _ = buf_writer.write(
                    match operation {
                        AlignmentOperation::Match => b"=",
                        AlignmentOperation::Subst => b"X",
                        AlignmentOperation::Insertion => b"I",
                        AlignmentOperation::Deletion => b"D",
                    }
                ).unwrap();
            });
            let _ = buf_writer.write(b"\n").unwrap();
        });
    });
}
