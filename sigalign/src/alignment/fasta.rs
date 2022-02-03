use crate::{Result, error_msg};
use super::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Aligner,
    Algorithms,
    AlignerInterface,
    LocalAligner,
    SemiGlobalAligner,
    Reference,
    SequenceProvider,
    LabelProvider,
};
use super::{
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    AlignmentLabeledResult,
    RecordAlignmentLabeledResult,
    FastaAlignmentResult,
    ReadAlignmentResult,
    AlignmentResult,
    RecordAlignmentResult,
    AnchorAlignmentResult,
    AlignmentPosition,
    AlignmentOperation,
    AlignmentCase,
};


use crate::util::{FastaReader};

use std::path::Path;
use std::io::{Write, Read};

impl Aligner {
    pub fn fasta_file_alignment<S, P>(&mut self, reference: &Reference<S>, fasta_file: P) -> Result<FastaAlignmentResult> where
        S: SequenceProvider,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_alignment_from_reader(reference, fasta_reader))
    }
    pub fn fasta_file_labeled_alignment<SL, P>(&mut self, reference: &Reference<SL>, fasta_file: P) -> Result<FastaAlignmentLabeledResult> where
       SL: SequenceProvider + LabelProvider,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_labeled_alignment_from_reader(reference, fasta_reader))
    }
    pub fn fasta_bytes_alignment<S>(&mut self, reference: &Reference<S>, fasta_bytes: &[u8]) -> FastaAlignmentResult where
        S: SequenceProvider,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_alignment_from_reader(reference, fasta_reader)
    }
    pub fn fasta_bytes_labeled_alignment<SL>(&mut self, reference: &Reference<SL>, fasta_bytes: &[u8]) -> FastaAlignmentLabeledResult where
      SL: SequenceProvider + LabelProvider,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_labeled_alignment_from_reader(reference, fasta_reader)
    }

    fn fasta_labeled_alignment_from_reader<R, SL>(&mut self, reference: &Reference<SL>, fasta_reader: FastaReader<R>) -> FastaAlignmentLabeledResult where
        R: Read,    
        SL: SequenceProvider + LabelProvider,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentLabeledResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.searchable(&query) {
                    Some(
                        ReadAlignmentLabeledResult {
                            read: label,
                            result: self.alignment(reference, &mut sequence_buffer, &query).to_labeled(reference),
                        }
                    )
                } else {
                    None
                }
            }).collect()
        )
    }
    fn fasta_alignment_from_reader<R, S>(&mut self, reference: &Reference<S>, fasta_reader: FastaReader<R>) -> FastaAlignmentResult where
        R: Read,    
        S: SequenceProvider,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.searchable(&query) {
                    Some(
                        ReadAlignmentResult {
                            read: label,
                            result: self.alignment(reference, &mut sequence_buffer, &query),
                        }
                    )
                } else {
                    None
                }
            }).collect()
        )
    }
    
    pub fn fasta_file_alignment_json_to_stream<W, P, S>(&mut self, reference: &Reference<S>, fasta_file: P, stream: W) -> Result<()> where
        W: Write,
        P: AsRef<Path> + std::fmt::Debug,
        S: SequenceProvider,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        self.write_fasta_alignment_json_from_reader(reference, fasta_reader, stream)
    }
    pub fn fasta_file_labeled_alignment_json_to_stream<W, P, SL>(&mut self, reference: &Reference<SL>, fasta_file: P, stream: W) -> Result<()> where
        W: Write,
        P: AsRef<Path> + std::fmt::Debug,
        SL: SequenceProvider + LabelProvider,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        self.write_fasta_labeled_alignment_json_from_reader(reference, fasta_reader, stream)
    }

    fn write_fasta_alignment_json_from_reader<R, W, S>(&mut self, reference: &Reference<S>, mut fasta_reader: FastaReader<R>, mut writer: W) -> Result<()> where
        R: Read,
        W: Write,
        S: SequenceProvider,
    {
        let mut sequence_buffer = reference.get_buffer();
        // First read
        writer.write(b"[")?;
        let mut need_first_written = true;
        while need_first_written {
            if let Some((label, query)) = fasta_reader.next() {
                if reference.searchable(&query) {
                    let read_alignment_result = ReadAlignmentResult {
                        read: label,
                        result: self.alignment(reference, &mut sequence_buffer, &query),
                    };
                    read_alignment_result.write_as_json(&mut writer);
                    need_first_written = false;
                }
            };
        }

        // Middle reads
        fasta_reader.into_iter().for_each(|(label, query)| {
            if reference.searchable(&query) {
                let read_alignment_result = ReadAlignmentResult {
                    read: label,
                    result: self.alignment(reference, &mut sequence_buffer, &query),
                };
                writer.write(b","); // Do not error check
                read_alignment_result.write_as_json(&mut writer);
            }
            // Ignore unsearchable query
        });

        // Last closing
        writer.write(b"]")?;
        writer.flush()?;

        Ok(())
    }
    fn write_fasta_labeled_alignment_json_from_reader<R, W, SL>(&mut self, reference: &Reference<SL>, mut fasta_reader: FastaReader<R>, mut writer: W) -> Result<()> where
        R: Read,
        W: Write,
        SL: SequenceProvider + LabelProvider,
    {
        let mut sequence_buffer = reference.get_buffer();
        // First read
        writer.write(b"[")?;
        let mut need_first_written = true;
        while need_first_written {
            if let Some((label, query)) = fasta_reader.next() {
                if reference.searchable(&query) {
                    let read_alignment_result = ReadAlignmentResult {
                        read: label,
                        result: self.alignment(reference, &mut sequence_buffer, &query),
                    };
                    read_alignment_result.write_as_json(&mut writer);
                    need_first_written = false;
                }
            };
        }

        // Middle reads
        fasta_reader.into_iter().for_each(|(label, query)| {
            if reference.searchable(&query) {
                let read_labeled_alignment_result = ReadAlignmentLabeledResult {
                    read: label,
                    result: self.alignment(reference, &mut sequence_buffer, &query).to_labeled(reference),
                };
                writer.write(b","); // Do not error check
                read_labeled_alignment_result.write_as_json(&mut writer);
            }
            // Ignore unsearchable query
        });

        // Last closing
        writer.write(b"]")?;
        writer.flush()?;

        Ok(())
    }
}