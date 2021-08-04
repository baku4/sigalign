//! Dropout alignment
pub mod alignment;
pub mod io;

/* BASIC TYPES */

/// Length of Sequnce: Same as memory bandwidth
pub type SequenceLength = usize;
/// Length of Operation: 32 bit
pub type OperationLength = u32;
/// Penalty(Score): Same as memory bandwidth
pub type Penalty = usize;

#[cfg(test)]
mod tests;