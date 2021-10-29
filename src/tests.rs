#![allow(dead_code, unused)]
use super::*;
use crate::core::*;
use crate::reference::*;
use crate::reference::sequence_provider::*;
use crate::algorithm::*;
use crate::aligner::*;

// Supply Functions
mod sample_data;
use sample_data::*;

mod standard_aligner;
use standard_aligner::*; 

// Test Main
mod test_alignment_algorithm;

mod test_sequence_provider;