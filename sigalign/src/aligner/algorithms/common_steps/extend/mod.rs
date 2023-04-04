use crate::results::AlignmentOperations;

/// Wavefront structure
mod wave_front;
pub use wave_front::{WaveFront, WaveEndPoint, WaveFrontScore, Components, Component, BackTraceMarker};

/// Extension
#[derive(Debug, Clone)]
pub struct Extension {
    pub penalty: u32,
    pub length: u32,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub reversed_operations: Vec<AlignmentOperations>,
}
