use crate::{Result};
use super::{
    ReferenceInterface,
};
use super::{
    Aligner,
    Reference,
    SequenceStorage,
    LabelStorage,
};
use super::{
    FastaAlignmentLabeledResult,
    ReadAlignmentLabeledResult,
    FastaAlignmentResult,
    ReadAlignmentResult,
};


use crate::util::{FastaReader};

use std::path::Path;
use std::io::{Write, Read};

/// Methods for alignment with FASTA
impl Aligner {
    /// Alignment FASTA file checking all query is supported type of reference.
    ///  - Unsupported type of queries are ignored.
    pub fn fasta_file_alignment<S, P>(&mut self, reference: &Reference<S>, fasta_file: P) -> Result<FastaAlignmentResult> where
        S: SequenceStorage,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_alignment_from_reader(reference, fasta_reader))
    }
    /// Alignment FASTA file without checking query.
    ///  - This method can make thread panic.
    ///  - Use if you are sure that the type of query is supported by reference.
    pub fn fasta_file_alignment_unchecked<S, P>(&mut self, reference: &Reference<S>, fasta_file: P) -> Result<FastaAlignmentResult> where
        S: SequenceStorage,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_alignment_from_reader_unchecked(reference, fasta_reader))
    }
    /// Alignment FASTA file without checking query.
    ///  - Unsupported type of queries are ignored.
    ///  - The output is labeled result.
    ///  - Only available when [SequenceStorage] of [Reference] is also [LabelStorage].
    pub fn fasta_file_labeled_alignment<SL, P>(&mut self, reference: &Reference<SL>, fasta_file: P) -> Result<FastaAlignmentLabeledResult> where
       SL: SequenceStorage + LabelStorage,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_labeled_alignment_from_reader(reference, fasta_reader))
    }
    /// Alignment FASTA file without checking query.
    ///  - This method can make thread panic.
    ///  - Use if you are sure that the type of query is supported by reference.
    ///  - The output is labeled result.
    ///  - Only available when [SequenceStorage] of [Reference] is also [LabelStorage].
    pub fn fasta_file_labeled_alignment_unchecked<SL, P>(&mut self, reference: &Reference<SL>, fasta_file: P) -> Result<FastaAlignmentLabeledResult> where
       SL: SequenceStorage + LabelStorage,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.fasta_labeled_alignment_from_reader_unchecked(reference, fasta_reader))
    }

    /// Alignment FASTA formatted bytes checking all query is supported type of reference.
    ///  - Unsupported type of queries are ignored.
    pub fn fasta_bytes_alignment<S>(&mut self, reference: &Reference<S>, fasta_bytes: &[u8]) -> FastaAlignmentResult where
        S: SequenceStorage,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_alignment_from_reader(reference, fasta_reader)
    }
    /// Alignment FASTA formatted bytes without checking query.
    /// - This method can make thread panic.
    ///  - Use if you are sure that the type of query is supported by reference.
    pub fn fasta_bytes_alignment_unchecked<S>(&mut self, reference: &Reference<S>, fasta_bytes: &[u8]) -> FastaAlignmentResult where
        S: SequenceStorage,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_alignment_from_reader_unchecked(reference, fasta_reader)
    }
    /// Alignment FASTA formatted bytes without checking query.
    ///  - Unsupported type of queries are ignored.
    ///  - The output is labeled result.
    ///  - Only available when [SequenceStorage] of [Reference] is also [LabelStorage].
    pub fn fasta_bytes_labeled_alignment<SL>(&mut self, reference: &Reference<SL>, fasta_bytes: &[u8]) -> FastaAlignmentLabeledResult where
      SL: SequenceStorage + LabelStorage,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_labeled_alignment_from_reader(reference, fasta_reader)
    }
    /// Alignment FASTA formatted bytes without checking query.
    ///  - This method can make thread panic.
    ///  - Use if you are sure that the type of query is supported by reference.
    ///  - The output is labeled result.
    ///  - Only available when [SequenceStorage] of [Reference] is also [LabelStorage].
    pub fn fasta_bytes_labeled_alignment_unchecked<SL>(&mut self, reference: &Reference<SL>, fasta_bytes: &[u8]) -> FastaAlignmentLabeledResult where
      SL: SequenceStorage + LabelStorage,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.fasta_labeled_alignment_from_reader_unchecked(reference, fasta_reader)
    }

    fn fasta_alignment_from_reader<R, S>(&mut self, reference: &Reference<S>, fasta_reader: FastaReader<R>) -> FastaAlignmentResult where
        R: Read,    
        S: SequenceStorage,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.searchable(&query) {
                    let result = self.alignment(reference, &mut sequence_buffer, &query);
                    if result.0.len() == 0 {
                        None
                    } else {
                        Some(
                            ReadAlignmentResult {
                                read: label,
                                result: result,
                            }
                        )
                    }
                } else {
                    None
                }
            }).collect()
        )
    }
    fn fasta_alignment_from_reader_unchecked<R, S>(&mut self, reference: &Reference<S>, fasta_reader: FastaReader<R>) -> FastaAlignmentResult where
        R: Read,    
        S: SequenceStorage,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                let result = self.alignment(reference, &mut sequence_buffer, &query);
                if result.0.len() == 0 {
                    None
                } else {
                    Some(
                        ReadAlignmentResult {
                            read: label,
                            result: result,
                        }
                    )
                }
            }).collect()
        )
    }
    fn fasta_labeled_alignment_from_reader<R, SL>(&mut self, reference: &Reference<SL>, fasta_reader: FastaReader<R>) -> FastaAlignmentLabeledResult where
        R: Read,    
        SL: SequenceStorage + LabelStorage,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentLabeledResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.searchable(&query) {
                    let result = self.alignment(reference, &mut sequence_buffer, &query);
                    if result.0.len() == 0 {
                        None
                    } else {
                        Some(
                            ReadAlignmentLabeledResult {
                                read: label,
                                result: result.to_labeled(reference),
                            }
                        )
                    }
                } else {
                    None
                }
            }).collect()
        )
    }
    fn fasta_labeled_alignment_from_reader_unchecked<R, SL>(&mut self, reference: &Reference<SL>, fasta_reader: FastaReader<R>) -> FastaAlignmentLabeledResult where
        R: Read,    
        SL: SequenceStorage + LabelStorage,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentLabeledResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                let result = self.alignment(reference, &mut sequence_buffer, &query);
                if result.0.len() == 0 {
                    None
                } else {
                    Some(
                        ReadAlignmentLabeledResult {
                            read: label,
                            result: result.to_labeled(reference),
                        }
                    )
                }
            }).collect()
        )
    }
    
    /// Alignment FASTA file and write `Json` result to stream.
    pub fn fasta_file_alignment_json_to_stream<W, P, S>(&mut self, reference: &Reference<S>, fasta_file: P, stream: W) -> Result<()> where
        W: Write,
        P: AsRef<Path> + std::fmt::Debug,
        S: SequenceStorage,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        self.write_fasta_alignment_json_from_reader(reference, fasta_reader, stream)
    }
    /// Alignment FASTA file and write `Json` result to stream.
    ///  - The output is labeled result.
    ///  - Only available when [SequenceStorage] of [Reference] is also [LabelStorage].
    pub fn fasta_file_labeled_alignment_json_to_stream<W, P, SL>(&mut self, reference: &Reference<SL>, fasta_file: P, stream: W) -> Result<()> where
        W: Write,
        P: AsRef<Path> + std::fmt::Debug,
        SL: SequenceStorage + LabelStorage,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        self.write_fasta_labeled_alignment_json_from_reader(reference, fasta_reader, stream)
    }

    fn write_fasta_alignment_json_from_reader<R, W, S>(&mut self, reference: &Reference<S>, mut fasta_reader: FastaReader<R>, mut writer: W) -> Result<()> where
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
        #[allow(unused_must_use)]
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
        SL: SequenceStorage + LabelStorage,
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
        #[allow(unused_must_use)]
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