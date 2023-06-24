use crate::{
    results::{
        labeled::{
            LabeledAlignmentResult,
        },
        fasta::{
            LabeledFastaAlignmentResult,
            LabeledReadAlignmentResult,
        },
    },
    reference::{
        Reference,
        pattern_index::PatternIndex,
        sequence_storage::SequenceStorage,
        extensions::LabelStorage,
    },
    utils::{
        FastaReader,
    },
};
use super::{
    Aligner,
    AlignmentMode,
    AllocationStrategy,
    AlignmentError,
};

impl<M, A> Aligner<M, A> where
    M: AlignmentMode,
    A: AllocationStrategy,
{
    // Query
    pub fn align_query_labeled<I, S> (
        &mut self,
        reference: &Reference<I, S>,
        query: &[u8],
    ) -> LabeledAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
    {
        let mut sequence_buffer = reference.get_sequence_buffer();
        let result = self.alignment(reference, &mut sequence_buffer, query);
        result.to_labeled_result_unchecked(reference)
    }
    // FASTA
    pub fn align_fasta_file_labeled<I, S, P> (
        &mut self,
        reference: &Reference<I, S>,
        file_path: P,
    ) -> Result<LabeledFastaAlignmentResult, AlignmentError> where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
        P: AsRef<std::path::Path>,
    {
        let fasta_reader = FastaReader::from_path(file_path)?;
        Ok(self.align_from_fasta_reader_labeled(reference, fasta_reader))
    }
    pub fn align_fasta_bytes_labeled<I, S> (
        &mut self,
        reference: &Reference<I, S>,
        fasta_bytes: &[u8],
    ) -> Result<LabeledFastaAlignmentResult, AlignmentError> where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        Ok(self.align_from_fasta_reader_labeled(reference, fasta_reader))
    }
    fn align_from_fasta_reader_labeled<I, S, R> (
        &mut self,
        reference: &Reference<I, S>,
        fasta_reader: FastaReader<R>,
    ) -> LabeledFastaAlignmentResult where
        I: PatternIndex,
        S: SequenceStorage + LabelStorage,
        R: std::io::Read,
    {
        let mut sequence_buffer = reference.get_sequence_buffer();
        LabeledFastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                let result = self.alignment(reference, &mut sequence_buffer, &query);
                let labeled_result = result.to_labeled_result_unchecked(reference);
                if labeled_result.result_counts() == 0 {
                    None
                } else {
                    Some(
                        LabeledReadAlignmentResult {
                            read: label,
                            result: labeled_result,
                        }
                    )
                }
            }).collect()
        )
    }
}