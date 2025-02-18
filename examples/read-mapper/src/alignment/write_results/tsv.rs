use std::io::Write;

use anyhow::Result;
use sigalign::results::{AlignmentOperation, LabeledQueryAlignment};

#[derive(Clone)]
pub struct TsvFormatter {
    itoa_buffer: itoa::Buffer,
}

impl TsvFormatter {
    pub fn new() -> Self {
        Self {
            itoa_buffer: itoa::Buffer::new(),
        }
    }
    pub fn write_header(&mut self, writer: &mut impl Write) -> Result<()> {
        writer.write_all(
            b"query_name\tstrand\ttarget_name\tpenalty\tlength\tquery_start\tquery_end\ttarget_start\ttarget_end\toperations\n"
        )?;
        Ok(())
    }
    pub fn write_tsv_record(
        &mut self,
        writer: &mut impl Write,
        labeled_query_alignment: &LabeledQueryAlignment,
        query_name: &str,
        is_forward: bool,
    ) -> Result<()> {
        for labeled_target_alignment in labeled_query_alignment
        .0.iter() {
            for alignment in labeled_target_alignment.alignments.iter() {
                writer.write_all(query_name.as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(
                    if is_forward { b"+" } else { b"-" }
                )?;
                writer.write_all(b"\t")?;
                writer.write_all(labeled_target_alignment.label.as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.penalty).as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.length).as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.position.query.0).as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.position.query.1).as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.position.target.0).as_bytes())?;
                writer.write_all(b"\t")?;
                writer.write_all(self.itoa_buffer.format(alignment.position.target.1).as_bytes())?;
                writer.write_all(b"\t")?;
                // (6) CIGAR
                for op in alignment.operations.iter() {
                    writer.write(
                        self.itoa_buffer.format(op.count).as_bytes()
                    )?;
                    writer.write(
                        match op.operation {
                            AlignmentOperation::Match => b"=",
                            AlignmentOperation::Subst => b"X",
                            AlignmentOperation::Insertion => b"I",
                            AlignmentOperation::Deletion => b"D",
                        }
                    )?;
                }
                writer.write_all(b"\n")?;
            }
        }
        Ok(())
    }
}
