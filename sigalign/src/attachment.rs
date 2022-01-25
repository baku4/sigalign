// Additional features for aligner
use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use crate::aligner::{
    Aligner,
    Algorithms,
    AlignerInterface,
    LocalAligner,
    SemiGlobalAligner,
    Reference,
    SequenceProvider,
};
use crate::result::{
    FastaAlignmentResult,
    ReadAlignmentResult,
};
use crate::util::{FastaReader};

use std::path::Path;
use std::io::{Write, Read};

impl Aligner {
    pub fn query_alignment_unchecked<S: SequenceProvider>(
        &mut self,
        reference: &Reference<S>,
        query: Sequence,
    ) -> AlignmentResult {
        let mut sequence_buffer = reference.get_buffer();
        match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
        }
    }
    pub fn query_alignment<S: SequenceProvider>(
        &mut self,
        reference: &Reference<S>,
        query: Sequence,
    ) -> Result<AlignmentResult> {
        let mut sequence_buffer = reference.get_buffer();
        if !reference.searchable(query) {
            error_msg!("Query contains unsearchable character")
        }
        Ok(match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
        })
    }
    pub fn fasta_file_alignment<S, P>(&mut self, reference: &Reference<S>, fasta_file: P) -> Result<FastaAlignmentResult> where
        S: SequenceProvider,
        P: AsRef<Path> + std::fmt::Debug,
    {
        let fasta_reader = FastaReader::from_file_path(fasta_file)?;
        Ok(self.alignment_from_fasta_reader(reference, fasta_reader))
    }
    pub fn fasta_bytes_alignment<S>(&mut self, reference: &Reference<S>, fasta_bytes: &[u8]) -> FastaAlignmentResult where
        S: SequenceProvider,
    {
        let fasta_reader = FastaReader::from_bytes(fasta_bytes);
        self.alignment_from_fasta_reader(reference, fasta_reader)
    }
    fn alignment_from_fasta_reader<R, S>(&mut self, reference: &Reference<S>, fasta_reader: FastaReader<R>) -> FastaAlignmentResult where
        S: SequenceProvider,
        R: Read,
    {
        let mut sequence_buffer = reference.get_buffer();
        FastaAlignmentResult(
            fasta_reader.into_iter().filter_map(|(label, query)| {
                if !reference.searchable(&query) {
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
}