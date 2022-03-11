// Additional features for aligner
use crate::{Result, error_msg};
use crate::core::{
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
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
    LabelProvider,
};
use crate::result::{
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

mod query;
mod fasta;
