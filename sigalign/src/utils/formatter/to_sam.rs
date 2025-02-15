use std::io::{self, BufWriter, Write};

use crate::{
    Reference,
    results::{
        QueryAlignment,
        AlignmentOperation,
    },
};

/// A writer that produces SAM records in a buffer.
pub struct SamWriter<W: Write> {
    buf_writer: BufWriter<W>,
    itoa_buffer: itoa::Buffer,
}

impl<W: Write> SamWriter<W> {
    /// Creates a new SamWriter from any type implementing `Write`.
    pub fn from_writer(writer: W) -> Self {
        SamWriter {
            buf_writer: BufWriter::new(writer),
            itoa_buffer: itoa::Buffer::new(),
        }
    }
    /// Writes a minimal SAM header.
    pub fn write_header(&mut self) -> Result<(), io::Error> {
        self.buf_writer.write_all(b"@HD\tVN:1.6\tSO:unsorted\n")?;
        Ok(())
    }
    /// Write the SAM record 
    pub fn write_query_alignment(
        &mut self,
        query_alignment: &QueryAlignment,
        qname: &str,
        is_forward: bool,
        reference: &Reference, // To parse the target label
    ) -> Result<(), io::Error> {
        for target_alignment in query_alignment.0.iter() {
            let target_label = reference.get_label_str(target_alignment.index).unwrap_or_default();
            for alignment in target_alignment.alignments.iter() {
                // (1) QNAME
                self.buf_writer.write(qname.as_bytes())?;
                // (2) FLAG
                self.buf_writer.write(if is_forward { b"\t0\t" } else { b"\t16\t" })?;
                // (3) RNAME
                self.buf_writer.write(target_label.as_bytes())?;
                self.buf_writer.write(b"\t")?;
                // (4) POS
                //   SAM is 1-based, so add 1 to the 0-based position.
                self.buf_writer.write(
                    self.itoa_buffer.format(alignment.position.target.0 + 1).as_bytes()
                )?;
                // (5) MAPQ: 255 to indicate the score is not assigned
                self.buf_writer.write(b"\t255\t")?;
                // (6) CIGAR
                for op in alignment.operations.iter() {
                    self.buf_writer.write(
                        self.itoa_buffer.format(op.count).as_bytes()
                    )?;
                    self.buf_writer.write(
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
                self.buf_writer.write(b"\t*\t0\t0\t*\t*\n")?;
            }
        }

        Ok(())
    }
}
