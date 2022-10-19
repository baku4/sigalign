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
    SequenceStorage,
    LabelStorage,
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
    pub fn query_alignment<S: SequenceStorage>(
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
    pub fn query_alignment_unchecked<S: SequenceStorage>(
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
    pub fn query_labeled_alignment<SL: SequenceStorage + LabelStorage>(
        &mut self,
        reference: &Reference<SL>,
        query: Sequence,
    ) -> Result<AlignmentLabeledResult> {
        let mut sequence_buffer = reference.get_buffer();
        if !reference.searchable(query) {
            error_msg!("Query contains unsearchable character")
        }
        let alignment_result = match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
        };
        Ok(alignment_result.to_labeled(reference))
    }
    pub fn query_labeled_alignment_unchecked<SL: SequenceStorage + LabelStorage>(
        &mut self,
        reference: &Reference<SL>,
        query: Sequence,
    ) -> AlignmentLabeledResult {
        let mut sequence_buffer = reference.get_buffer();
        let alignment_result = match &mut self.algorithms {
            Algorithms::SemiGlobal(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
            Algorithms::Local(aligner) => aligner.alignment(reference, &mut sequence_buffer, query),
        };
        alignment_result.to_labeled(reference)
    }
}
