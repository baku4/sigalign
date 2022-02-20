use super::{
    Result, error_msg,
	Penalties, PRECISION_SCALE, Cutoff, MinPenaltyForPattern,
	AlignmentResult, RecordAlignmentResult, AnchorAlignmentResult, AlignmentPosition, AlignmentOperation, AlignmentCase,
    Sequence,
    ReferenceInterface, SequenceBuffer, PatternLocation,
};
use super::{
    Reference, SequenceProvider, JoinedSequence,
    SequenceType, PatternFinder,
    Serializable,
    LabelProvider,
    ReverseComplement,
};

mod fasta_path_finder;

use std::path::{Path, PathBuf};

pub fn path_to_string<P>(file_path: P) -> Result<String> where
    P: AsRef<Path> + std::fmt::Debug,
{
    match file_path.as_ref().canonicalize()?.to_str() {
        Some(v) => Ok(v.to_string()),
        None => error_msg!("Invalid file path"),
    }
}