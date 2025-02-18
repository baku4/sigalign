use std::io::{self, Write};

use crate::{
    results::{
        AlignmentOperation, LabeledQueryAlignment, QueryAlignment
    }, Reference
};

/// A formatter that writes SAM records.
#[derive(Clone)]
pub struct SamFormatter {
    itoa_buffer: itoa::Buffer,
}
impl SamFormatter {
    pub fn new() -> Self {
        Self {
            itoa_buffer: itoa::Buffer::new(),
        }
    }
    pub fn write_hd_header(&self, writer: &mut impl Write) -> Result<(), io::Error> {
        writer.write_all(b"@HD\tVN:1.6\tSO:unsorted\n")?;
        Ok(())
    }
    pub fn write_sq_header(
        &self,
        writer: &mut impl Write,
        sn: &str,
        ln: &u32,
    ) -> Result<(), io::Error> {
        writer.write_all(format!("@SQ\tSN:{}\tLN:{}\n", sn, ln).as_bytes())?;
        Ok(())
    }
    pub fn write_query_alignment(
        &mut self,
        writer: &mut impl Write,
        query_alignment: &QueryAlignment,
        qname: &str,
        is_forward: bool,
        reference: &Reference, // To parse the target label
    ) -> Result<(), io::Error> {
        for target_alignment in query_alignment.0.iter() {
            let target_label = reference.get_label_str(target_alignment.index).unwrap_or_default();
            for alignment in target_alignment.alignments.iter() {
                // (1) QNAME
                writer.write(qname.as_bytes())?;
                // (2) FLAG
                writer.write(if is_forward { b"\t0\t" } else { b"\t16\t" })?;
                // (3) RNAME
                writer.write(target_label.as_bytes())?;
                writer.write(b"\t")?;
                // (4) POS
                //   SAM is 1-based, so add 1 to the 0-based position.
                writer.write(
                    self.itoa_buffer.format(alignment.position.target.0 + 1).as_bytes()
                )?;
                // (5) MAPQ: 255 to indicate the score is not assigned
                writer.write(b"\t255\t")?;
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
                // (7) RNEXT
                // (8) PNEXT
                // (9) TLEN
                // (10) SEQ
                // (11) QUAL
                // For a minimal single-end record (no mate information, no sequence/qual data)
                writer.write(b"\t*\t0\t0\t*\t*\n")?;
            }
        }

        Ok(())
    }
    // TODO: Remove duplicated code
    pub fn write_labeled_query_alignment(
        &mut self,
        writer: &mut impl Write,
        labeled_query_alignment: &LabeledQueryAlignment,
        qname: &str,
        is_forward: bool,
    ) -> Result<(), io::Error> {
        for labeled_target_alignment in labeled_query_alignment.0.iter() {
            for alignment in labeled_target_alignment.alignments.iter() {
                // (1) QNAME
                writer.write(qname.as_bytes())?;
                // (2) FLAG
                writer.write(if is_forward { b"\t0\t" } else { b"\t16\t" })?;
                // (3) RNAME
                writer.write(labeled_target_alignment.label.as_bytes())?;
                writer.write(b"\t")?;
                // (4) POS
                //   SAM is 1-based, so add 1 to the 0-based position.
                writer.write(
                    self.itoa_buffer.format(alignment.position.target.0 + 1).as_bytes()
                )?;
                // (5) MAPQ: 255 to indicate the score is not assigned
                writer.write(b"\t255\t")?;
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
                // (7) RNEXT
                // (8) PNEXT
                // (9) TLEN
                // (10) SEQ
                // (11) QUAL
                // For a minimal single-end record (no mate information, no sequence/qual data)
                writer.write(b"\t*\t0\t0\t*\t*\n")?;
            }
        }

        Ok(())
    }
    // TODO: Remove duplicated code
    pub fn write_labeled_query_alignment_with_hclip(
        &mut self,
        writer: &mut impl Write,
        labeled_query_alignment: &LabeledQueryAlignment,
        qname: &str,
        is_forward: bool,
        query_length: u32,
    ) -> Result<(), io::Error> {
        for labeled_target_alignment in labeled_query_alignment.0.iter() {
            for alignment in labeled_target_alignment.alignments.iter() {
                // (1) QNAME
                writer.write(qname.as_bytes())?;
                // (2) FLAG
                writer.write(if is_forward { b"\t0\t" } else { b"\t16\t" })?;
                // (3) RNAME
                writer.write(labeled_target_alignment.label.as_bytes())?;
                writer.write(b"\t")?;
                // (4) POS
                //   SAM is 1-based, so add 1 to the 0-based position.
                writer.write(
                    self.itoa_buffer.format(alignment.position.target.0 + 1).as_bytes()
                )?;
                // (5) MAPQ: 255 to indicate the score is not assigned
                writer.write(b"\t255\t")?;
                // (6) CIGAR
                let lclip_size = alignment.position.query.0;
                if lclip_size != 0 {
                    writer.write(
                        self.itoa_buffer.format(lclip_size).as_bytes()
                    )?;
                    writer.write(b"H")?;
                }
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
                let rclip_size = query_length - alignment.position.query.1;
                if rclip_size != 0 {
                    writer.write(
                        self.itoa_buffer.format(rclip_size).as_bytes()
                    )?;
                    writer.write(b"H")?;
                }

                // (7) RNEXT
                // (8) PNEXT
                // (9) TLEN
                // (10) SEQ
                // (11) QUAL
                // For a minimal single-end record (no mate information, no sequence/qual data)
                writer.write(b"\t*\t0\t0\t*\t*\n")?;
            }
        }

        Ok(())
    }
}
