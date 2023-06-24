use crate::results::{
    AlignmentResult,
    fasta::{FastaAlignmentResult, ReadAlignmentResult},
};
use crate::reference::{
    Reference,
    pattern_index::PatternIndex,
    sequence_storage::SequenceStorage,
};
use crate::utils::FastaReader;
use super::{Aligner, AlignmentMode, AllocationStrategy};

use thiserror::Error;
#[derive(Debug, Error)]
pub enum AlignmentError {
    #[error("The query has unsupported character of reference")]
    UnsupportedQuery,
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    // Query
    pub fn align_query<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        query: &[u8],
    ) -> AlignmentResult {
        let mut sequence_buffer = reference.get_sequence_buffer();
        let result = self.alignment(reference, &mut sequence_buffer, query);
        result
    }
    // FASTA
    pub fn align_fasta_file<I, S, P> (
        &mut self,
        reference: &Reference<I, S>,
        file_path: P,
    ) -> Result<FastaAlignmentResult, AlignmentError> where
        I: PatternIndex,
        S: SequenceStorage,
        P: AsRef<std::path::Path>,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        Ok(self.align_from_fasta_reader(reference, fasta_reader))
    }
    pub fn align_fasta_bytes<I: PatternIndex, S: SequenceStorage> (
        &mut self,
        reference: &Reference<I, S>,
        fasta_bytes: &[u8],
    ) -> Result<FastaAlignmentResult, AlignmentError> {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        Ok(self.align_from_fasta_reader(reference, fasta_reader))
    }
    fn align_from_fasta_reader<I, S, R> (
        &mut self,
        reference: &Reference<I, S>,
        fasta_reader: FastaReader<R>,
    ) -> FastaAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage,
        R: std::io::Read,
    {
        let mut sequence_buffer = reference.get_sequence_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                let result = self.alignment(reference, &mut sequence_buffer, &query);
                if result.result_counts() == 0 {
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
}

// Features
mod labeled;