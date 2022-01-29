#![allow(dead_code, unused)]
use super::*;
use crate::core::*;
use crate::util::*;
use crate::reference::*;
use crate::reference::sequence_provider::*;
use crate::aligner::*;
use crate::result::*;
use crate::alignment::*;
use crate::builder::*;

// Aligner to verifying result
mod standard_aligner;
use standard_aligner::*;

// Supply Functions
#[cfg(test)]
pub mod sample_data;
#[cfg(test)]
use sample_data::*;

// Test Main
#[cfg(test)]
mod print_alignment_result_to_cmp;
#[cfg(test)]
mod test_sequence_provider;
