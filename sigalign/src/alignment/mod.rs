// Additional features for aligner
use crate::core::{
    Sequence,
    ReferenceInterface,
};
// TODO: Delete unused_imports
#[allow(unused_imports)]
use crate::aligner::{
    Aligner,
    Algorithms,
    AlignerInterface,
    LocalAligner,
    SemiGlobalAligner,
    Reference,
    SequenceStorage,
    LabelStorage,
};
// TODO: Delete unused_imports
#[allow(unused_imports)]
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
