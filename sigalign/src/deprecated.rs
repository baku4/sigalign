//! Dropout alignment
// pub mod alignment_dep;
// pub mod reference;s
pub mod alignment;
pub mod io;
pub mod utils;
pub mod database;

/* BASIC TYPES */

/// Length of Sequnce: Same as memory bandwidth
pub type SequenceLength = usize;
/// Length of Operation: 32 bit
pub type OperationLength = u32;
/// Penalty(Score): Same as memory bandwidth
pub type Penalty = usize;

use anyhow::{Result, Error, anyhow};

// #[cfg(test)]
// mod tests;