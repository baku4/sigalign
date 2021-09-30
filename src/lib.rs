use anyhow::{Result, bail as error_msg};

// Core algorithm
mod core;

// Aligner
mod aligner;

// Reference
mod reference;

#[doc(hidden)]
pub mod deprecated;