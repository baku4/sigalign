use anyhow::{Result, bail as error_msg};

#[doc(hidden)]
mod core;
#[doc(hidden)]
mod algorithm;

mod alignment_result;

// Reference
mod reference;

// Aligner
mod aligner;



#[doc(hidden)]
pub mod deprecated;