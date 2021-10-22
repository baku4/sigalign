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

// Result Interpreter
mod result_Interpreter;


#[doc(hidden)]
pub mod deprecated;