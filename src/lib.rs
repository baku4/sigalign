use anyhow::{Result, bail as error_msg};

// Core Logic
mod core;

// Aligner
mod aligner;

mod utils;

#[doc(hidden)]
pub mod deprecated;