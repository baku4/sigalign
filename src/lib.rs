use anyhow::{Result, bail as error_msg};

// Aligner
mod aligner;
// Core algorithm
mod core;

#[doc(hidden)]
pub mod deprecated;