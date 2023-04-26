use super::{Result, error_msg};

use std::{
    path::PathBuf,
    time::Instant,
};

use clap::{
    Command,
    arg,
    ArgMatches,
    value_parser,
};

mod reference_path;
pub use reference_path::ReferencePaths;

#[cfg(not(feature = "idx_fa"))]
mod default_reference_app;
#[cfg(not(feature = "idx_fa"))]
pub use default_reference_app::{
    ReferenceApp,
    SigReferenceWrapper,
    InnerReference,
};

#[cfg(feature = "idx_fa")]
mod indexed_fasta_reference;
#[cfg(feature = "idx_fa")]
pub use indexed_fasta_reference::{
    ReferenceApp,
    Reference,
    InnerReference,
};
