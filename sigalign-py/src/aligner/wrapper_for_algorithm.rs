use pyo3::prelude::*;
use pyo3::exceptions::{
    PyException,
    PyValueError,
};
use sigalign::{
    Aligner,
    algorithms::{
        Local, LocalWithLimit, LocalWithChunk,
        SemiGlobal, SemiGlobalWithLimit, SemiGlobalWithChunk,
    },
    results::{
        QueryAlignment, TargetAlignment, Alignment,
    }
};

use crate::reference::PyReference;

pub enum AlignerWrapper {
    Local(Aligner<Local>),
    LocalWithLimit(Aligner<LocalWithLimit>),
    LocalWithChunk(Aligner<LocalWithChunk>),
    SemiGlobal(Aligner<SemiGlobal>),
    SemiGlobalWithLimit(Aligner<SemiGlobalWithLimit>),
    SemiGlobalWithChunk(Aligner<SemiGlobalWithChunk>),
}

fn map_params_err(err: sigalign::algorithms::ParamsError) -> PyErr {
    PyValueError::new_err(err.to_string())
}

impl AlignerWrapper {
    // New
    pub fn new(
        px: u32,
        po: u32,
        pe: u32,
        minl: u32,
        maxp: f32,
        use_local_mode: bool,
        use_limitation: Option<u32>,
        use_chunked: Option<(u32, u32)>,
    ) -> PyResult<Self> {
        if use_local_mode {
            if let Some(limit) = use_limitation {
                let algorithm = LocalWithLimit::new(px, po, pe, minl, maxp, limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::LocalWithLimit(Aligner::from(algorithm)))
            } else if let Some((chunk_size, chunk_limit)) = use_chunked {
                let algorithm = LocalWithChunk::new(px, po, pe, minl, maxp, chunk_size, chunk_limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::LocalWithChunk(Aligner::from(algorithm)))
            } else {
                let algorithm = Local::new(px, po, pe, minl, maxp).map_err(map_params_err)?;
                Ok(AlignerWrapper::Local(Aligner::from(algorithm)))
            }
        } else {
            if let Some(limit) = use_limitation {
                let algorithm = SemiGlobalWithLimit::new(px, po, pe, minl, maxp, limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::SemiGlobalWithLimit(Aligner::from(algorithm)))
            } else if let Some((chunk_size, chunk_limit)) = use_chunked {
                let algorithm = SemiGlobalWithChunk::new(px, po, pe, minl, maxp, chunk_size, chunk_limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::SemiGlobalWithChunk(Aligner::from(algorithm)))
            } else {
                let algorithm = SemiGlobal::new(px, po, pe, minl, maxp).map_err(map_params_err)?;
                Ok(AlignerWrapper::SemiGlobal(Aligner::from(algorithm)))
            }
        }
    }

    // Getters
    pub fn get_mismatch_penalty(&self) -> u32 {
        match self {
            AlignerWrapper::Local(v) => v.get_mismatch_penalty(),
            AlignerWrapper::LocalWithLimit(v) => v.get_mismatch_penalty(),
            AlignerWrapper::LocalWithChunk(v) => v.get_mismatch_penalty(),
            AlignerWrapper::SemiGlobal(v) => v.get_mismatch_penalty(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_mismatch_penalty(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_mismatch_penalty(),
        }
    }
    pub fn get_gap_open_penalty(&self) -> u32 {
        match self {
            AlignerWrapper::Local(v) => v.get_gap_open_penalty(),
            AlignerWrapper::LocalWithLimit(v) => v.get_gap_open_penalty(),
            AlignerWrapper::LocalWithChunk(v) => v.get_gap_open_penalty(),
            AlignerWrapper::SemiGlobal(v) => v.get_gap_open_penalty(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_gap_open_penalty(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_gap_open_penalty(),
        }
    }
    pub fn get_gap_extend_penalty(&self) -> u32 {
        match self {
            AlignerWrapper::Local(v) => v.get_gap_extend_penalty(),
            AlignerWrapper::LocalWithLimit(v) => v.get_gap_extend_penalty(),
            AlignerWrapper::LocalWithChunk(v) => v.get_gap_extend_penalty(),
            AlignerWrapper::SemiGlobal(v) => v.get_gap_extend_penalty(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_gap_extend_penalty(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_gap_extend_penalty(),
        }
    }
    pub fn get_pattern_size(&self) -> u32 {
        match self {
            AlignerWrapper::Local(v) => v.get_pattern_size(),
            AlignerWrapper::LocalWithLimit(v) => v.get_pattern_size(),
            AlignerWrapper::LocalWithChunk(v) => v.get_pattern_size(),
            AlignerWrapper::SemiGlobal(v) => v.get_pattern_size(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_pattern_size(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_pattern_size(),
        }
    }
    pub fn get_minimum_length(&self) -> u32 {
        match self {
            AlignerWrapper::Local(v) => v.get_minimum_length(),
            AlignerWrapper::LocalWithLimit(v) => v.get_minimum_length(),
            AlignerWrapper::LocalWithChunk(v) => v.get_minimum_length(),
            AlignerWrapper::SemiGlobal(v) => v.get_minimum_length(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_minimum_length(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_minimum_length(),
        }
    }
    pub fn get_maximum_penalty_per_length(&self) -> f32 {
        match self {
            AlignerWrapper::Local(v) => v.get_maximum_penalty_per_length(),
            AlignerWrapper::LocalWithLimit(v) => v.get_maximum_penalty_per_length(),
            AlignerWrapper::LocalWithChunk(v) => v.get_maximum_penalty_per_length(),
            AlignerWrapper::SemiGlobal(v) => v.get_maximum_penalty_per_length(),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.get_maximum_penalty_per_length(),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.get_maximum_penalty_per_length(),
        }
    }
    // TODO: Add limitation and chunk getters
    pub fn as_str(&self) -> &str {
        match self {
            AlignerWrapper::Local(_) => "Local",
            AlignerWrapper::LocalWithLimit(_) => "LocalWithLimit",
            AlignerWrapper::LocalWithChunk(_) => "LocalWithChunk",
            AlignerWrapper::SemiGlobal(_) => "SemiGlobal",
            AlignerWrapper::SemiGlobalWithLimit(_) => "SemiGlobalWithLimit",
            AlignerWrapper::SemiGlobalWithChunk(_) => "SemiGlobalWithChunk",
        }
    }

    // Perform Alignment
    pub fn align_query(&mut self, query: &str, reference: &PyReference) -> QueryAlignment {
        let reference = reference.as_ref();
        let query_alignment = match self {
            AlignerWrapper::Local(v) => v.align(query.as_bytes(), reference),
            AlignerWrapper::LocalWithLimit(v) => v.align(query.as_bytes(), reference),
            AlignerWrapper::LocalWithChunk(v) => v.align(query.as_bytes(), reference),
            AlignerWrapper::SemiGlobal(v) => v.align(query.as_bytes(), reference),
            AlignerWrapper::SemiGlobalWithLimit(v) => v.align(query.as_bytes(), reference),
            AlignerWrapper::SemiGlobalWithChunk(v) => v.align(query.as_bytes(), reference),
        };
        query_alignment
    }
}