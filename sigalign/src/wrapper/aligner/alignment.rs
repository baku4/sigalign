use crate::core::BufferedPatternSearch;
use crate::results::{
    AlignmentResult,
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
        FastaReverseComplementAlignmentResult,
        ReadReverseComplementAlignmentResult,
    },
};
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};
use crate::utils::{FastaReader, reverse_complement_of_dna};
use super::{DefaultAligner, SelfDescAligner};

use thiserror::Error;
#[derive(Debug, Error)]
pub enum DefaultAlignmentError {
    #[error("The query has unsupported character of reference")]
    UnsupportedQuery,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl DefaultAligner {
    // One query
    pub fn align_query<R: BufferedPatternSearch>(&mut self, reference: &R, query: &[u8]) -> Result<AlignmentResult, DefaultAlignmentError> {
        if !reference.is_alignable(query) {
            return Err(DefaultAlignmentError::UnsupportedQuery)
        }
        let mut sequence_buffer = reference.get_buffer();
        Ok(self.align_query_unchecked_with_sequence_buffer::<R>(reference, &mut sequence_buffer, query))
    }
    pub fn align_query_unchecked_with_sequence_buffer<R: BufferedPatternSearch>(
        &mut self,
        reference: &R,
        sequence_buffer: &mut R::Buffer,
        query: &[u8],
    ) -> AlignmentResult {
        match &mut self.inner {
            SelfDescAligner::Local(v) => v.alignment(reference, sequence_buffer, query),
            SelfDescAligner::SemiGlobal(v) => v.alignment(reference, sequence_buffer, query),
        }
    }
    // FASTA
    pub fn align_fasta_file<R, P>(&mut self, reference: &R, file_path: P) -> Result<FastaAlignmentResult, DefaultAlignmentError> where
        R: BufferedPatternSearch,
        P: AsRef<std::path::Path>,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        Ok(self.align_from_fasta_reader(reference, fasta_reader))
    }
    pub fn align_fasta_bytes<R, P>(&mut self, reference: &R, bytes: &[u8]) -> FastaAlignmentResult where
        R: BufferedPatternSearch,
    {
        let fasta_reader = FastaReader::from_bytes(bytes);
        self.align_from_fasta_reader(reference, fasta_reader)
    }
    // FASTA with Rc
    pub fn align_fasta_file_with_rc_dna<R, P>(&mut self, reference: &R, file_path: P) -> Result<FastaReverseComplementAlignmentResult, DefaultAlignmentError> where
        R: BufferedPatternSearch,
        P: AsRef<std::path::Path>,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        Ok(self.align_from_fasta_reader_with_rc_dna(reference, fasta_reader))
    }
    pub fn align_fasta_bytes_with_rc_dna<R, P>(&mut self, reference: &R, bytes: &[u8]) -> FastaReverseComplementAlignmentResult where
        R: BufferedPatternSearch,
    {
        let fasta_reader = FastaReader::from_bytes(bytes);
        self.align_from_fasta_reader_with_rc_dna(reference, fasta_reader)
    }

    fn align_from_fasta_reader<R1, R2>(&mut self, reference: &R1, fasta_reader: FastaReader<R2>) -> FastaAlignmentResult where
        R1: BufferedPatternSearch,
        R2: std::io::Read,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.is_alignable(&query) {
                    let result = self.align_query_unchecked_with_sequence_buffer(reference, &mut sequence_buffer, &query);
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
    fn align_from_fasta_reader_with_rc_dna<R1, R2>(
        &mut self,
        reference: &R1,
        fasta_reader: FastaReader<R2>,
    ) -> FastaReverseComplementAlignmentResult where
        R1: BufferedPatternSearch,
        R2: std::io::Read,
    {
        let mut sequence_buffer = reference.get_buffer();
        let mut results = Vec::new(); //TODO: Apply cap
        fasta_reader.into_iter().for_each(|(label, query)| {
            if reference.is_alignable(&query) {
                // Forward
                let result = self.align_query_unchecked_with_sequence_buffer(reference, &mut sequence_buffer, &query);
                if result.0.len() != 0 {
                    results.push(
                        ReadReverseComplementAlignmentResult {
                            read: label.clone(),
                            is_forward: true,
                            result: result,
                        }
                    )
                };
                // Reverse
                let rc_query = reverse_complement_of_dna(&query);
                let result = self.align_query_unchecked_with_sequence_buffer(reference, &mut sequence_buffer, &rc_query);
                if result.0.len() != 0 {
                    results.push(
                        ReadReverseComplementAlignmentResult {
                            read: label,
                            is_forward: false,
                            result: result,
                        }
                    )
                };
            }
        });

        FastaReverseComplementAlignmentResult(results)
    }
}