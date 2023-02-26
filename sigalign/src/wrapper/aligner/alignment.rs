use crate::{core::ReferenceInterface, aligner::AlignerInterface};
use crate::results::{
    AlignmentResult,
    fasta::{FastaAlignmentResult, ReadAlignmentResult},
};
use crate::utils::FastaReader;
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
    pub fn align_query<R: ReferenceInterface>(&mut self, reference: &R, query: &[u8]) -> Result<AlignmentResult, DefaultAlignmentError> {
        if !reference.is_valid(query) {
            return Err(DefaultAlignmentError::UnsupportedQuery)
        }
        let mut sequence_buffer = reference.get_buffer();
        Ok(self.align_query_unchecked_with_sequence_buffer::<R>(reference, &mut sequence_buffer, query))
    }
    pub fn align_fasta_file<R, P>(&mut self, reference: &R, file_path: P) -> Result<FastaAlignmentResult, DefaultAlignmentError> where
        R: ReferenceInterface,
        P: AsRef<std::path::Path>,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        Ok(self.align_from_fasta_reader(reference, fasta_reader))

    }
    pub fn align_fasta_bytes<R, P>(&mut self, reference: &R, bytes: &[u8]) -> FastaAlignmentResult where
        R: ReferenceInterface,
    {
        let fasta_reader = FastaReader::from_bytes(bytes);
        self.align_from_fasta_reader(reference, fasta_reader)

    }

    fn align_from_fasta_reader<R1, R2>(&mut self, reference: &R1, fasta_reader: FastaReader<R2>) -> FastaAlignmentResult where
        R1: ReferenceInterface,
        R2: std::io::Read,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if reference.is_valid(&query) {
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
    fn align_query_unchecked_with_sequence_buffer<R: ReferenceInterface>(
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
}