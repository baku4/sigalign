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
mod in_memory_reference;
#[cfg(not(feature = "idx_fa"))]
pub use in_memory_reference::{
    ReferenceApp,
    Reference,
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
