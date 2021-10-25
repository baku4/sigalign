use anyhow::{Result, bail as error_msg};
use serde::{Serialize, Deserialize, de::DeserializeOwned};

#[doc(hidden)]
// Core
mod core;
#[doc(hidden)]
// Algorithm
mod algorithm;

// Reference
mod reference;
// Aligner
mod aligner;

#[doc(hidden)]
pub mod deprecated;

#[cfg(test)]
mod tests;