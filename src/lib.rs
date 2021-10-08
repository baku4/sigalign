use anyhow::{Result, bail as error_msg};

#[doc(hidden)]
mod core;
#[doc(hidden)]
mod algorithm;

// Aligner
mod aligner;

// Reference
mod reference;

#[doc(hidden)]
pub mod deprecated;