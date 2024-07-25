use std::io::Read;

use pyo3::prelude::*;
use pyo3::exceptions::{
    PyException,
    PyValueError,
};
use sigalign::algorithms::Algorithm;
use sigalign::{
    Aligner, Reference,
    algorithms::{
        Local, LocalWithLimit, LocalWithChunk,
        SemiGlobal, SemiGlobalWithLimit, SemiGlobalWithChunk,
    },
    results::{
        QueryAlignment, TargetAlignment, Alignment,
    }
};
use sigalign_utils::sequence_reader::{IdRecord, SeqRecord};
use sigalign_utils::sequence_reader::{
    fasta::FastaReader,
};

use crate::reference::PyReference;
use crate::results::{PyFastaAlignment, PyQueryAlignment, PyReadAlignment};

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
        use_limit: Option<u32>,
        use_chunk: Option<(u32, u32)>,
    ) -> PyResult<Self> {
        if use_local_mode {
            if let Some(limit) = use_limit {
                let algorithm = LocalWithLimit::new(px, po, pe, minl, maxp, limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::LocalWithLimit(Aligner::from(algorithm)))
            } else if let Some((chunk_size, chunk_limit)) = use_chunk {
                let algorithm = LocalWithChunk::new(px, po, pe, minl, maxp, chunk_size, chunk_limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::LocalWithChunk(Aligner::from(algorithm)))
            } else {
                let algorithm = Local::new(px, po, pe, minl, maxp).map_err(map_params_err)?;
                Ok(AlignerWrapper::Local(Aligner::from(algorithm)))
            }
        } else {
            if let Some(limit) = use_limit {
                let algorithm = SemiGlobalWithLimit::new(px, po, pe, minl, maxp, limit).map_err(map_params_err)?;
                Ok(AlignerWrapper::SemiGlobalWithLimit(Aligner::from(algorithm)))
            } else if let Some((chunk_size, chunk_limit)) = use_chunk {
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
    #[inline]
    pub fn align_query(
        &mut self,
        query: &[u8],
        reference: &PyReference,
        with_label: bool,
    ) -> PyQueryAlignment {
        let reference = reference.as_ref();
        let query_alignment = match self {
            AlignerWrapper::Local(v) => align_query_with_core_aligner(v, query, reference),
            AlignerWrapper::LocalWithLimit(v) => align_query_with_core_aligner(v, query, reference),
            AlignerWrapper::LocalWithChunk(v) => align_query_with_core_aligner(v, query, reference),
            AlignerWrapper::SemiGlobal(v) => align_query_with_core_aligner(v, query, reference),
            AlignerWrapper::SemiGlobalWithLimit(v) => align_query_with_core_aligner(v, query, reference),
            AlignerWrapper::SemiGlobalWithChunk(v) => align_query_with_core_aligner(v, query, reference),
        };
        let py_query_alignment = if with_label {
            let labeled_query_alignment = reference.label_query_alignment(query_alignment);
            PyQueryAlignment::from(labeled_query_alignment)
        } else {
            PyQueryAlignment::from(query_alignment)
        };
        py_query_alignment
    }
    pub fn align_fasta_file(
        &mut self,
        reference: &PyReference,
        file_path: &str,
        with_label: bool,
        with_reverse_complementary: bool,
        checking_signals: bool,
    ) -> PyResult<PyFastaAlignment> {
        let mut fasta_reader = FastaReader::from_path(file_path)?;
        if checking_signals {
            self.align_fasta_with_checking_signals(reference, &mut fasta_reader, with_label, with_reverse_complementary)
        } else {
            self.align_fasta_without_checking_signals(reference, &mut fasta_reader, with_label, with_reverse_complementary)
        }
    }
    pub fn align_fasta_bytes(
        &mut self,
        reference: &PyReference,
        fasta_bytes: &[u8],
        with_label: bool,
        with_reverse_complementary: bool,
        checking_signals: bool,
    ) -> PyResult<PyFastaAlignment> {
        let mut fasta_reader = FastaReader::new(fasta_bytes);
        if checking_signals {
            self.align_fasta_with_checking_signals(reference, &mut fasta_reader, with_label, with_reverse_complementary)
        } else {
            self.align_fasta_without_checking_signals(reference, &mut fasta_reader, with_label, with_reverse_complementary)
        }
    }
    fn align_fasta_without_checking_signals<R: Read>(
        &mut self,
        reference: &PyReference,
        fasta_reader: &mut FastaReader<R>,
        with_label: bool,
        with_reverse_complementary: bool,
    ) -> PyResult<PyFastaAlignment> {
        match self {
            AlignerWrapper::Local(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
            AlignerWrapper::LocalWithLimit(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
            AlignerWrapper::LocalWithChunk(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
            AlignerWrapper::SemiGlobal(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
            AlignerWrapper::SemiGlobalWithLimit(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
            AlignerWrapper::SemiGlobalWithChunk(v) => {
                Ok(align_fasta_with_core_aligner(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary))
            },
        }
    }
    fn align_fasta_with_checking_signals<R: Read>(
        &mut self,
        reference: &PyReference,
        fasta_reader: &mut FastaReader<R>,
        with_label: bool,
        with_reverse_complementary: bool,
    ) -> PyResult<PyFastaAlignment> {
        match self {
            AlignerWrapper::Local(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
            AlignerWrapper::LocalWithLimit(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
            AlignerWrapper::LocalWithChunk(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
            AlignerWrapper::SemiGlobal(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
            AlignerWrapper::SemiGlobalWithLimit(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
            AlignerWrapper::SemiGlobalWithChunk(v) => {
                align_fasta_with_core_aligner_checking_signals(v, fasta_reader, reference.as_ref(), with_label, with_reverse_complementary)
            },
        }
    }
}

// Alignment Helpers
// - For query
#[inline]
fn align_query_with_core_aligner<A: Algorithm>(
    aligner: &mut Aligner<A>,
    query: &[u8],
    reference: &Reference,
) -> QueryAlignment {
    aligner.align(query, reference)
}
// - For FASTA
#[inline]
fn align_fasta_with_core_aligner<A: Algorithm, R: Read>(
    aligner: &mut Aligner<A>,
    fasta_reader: &mut FastaReader<R>,
    reference: &Reference,
    with_label: bool,
    with_reverse_complementary: bool,
) -> PyFastaAlignment {
    let mut py_read_alignments = Vec::new();

    let mut query_buffer = Vec::new();
    let mut label_buffer = String::new();
    while let Some(mut record) = fasta_reader.next() {
        query_buffer.clear();
        label_buffer.clear();
        record.extend_seq_buf(&mut query_buffer);
        record.extend_id_string(&mut label_buffer);

        let query_alignment = aligner.align(&query_buffer, reference);
        let py_query_alignmnet = if with_label {
            let labeled_query_alignment = reference.label_query_alignment(query_alignment);
            PyQueryAlignment::from(labeled_query_alignment)
        } else {
            PyQueryAlignment::from(query_alignment)
        };
        let py_read_alignment = PyReadAlignment{
            read: label_buffer.clone(),
            is_forward: true,
            result: py_query_alignmnet,
        };
        py_read_alignments.push(py_read_alignment);

        if with_reverse_complementary {
            query_buffer.reverse();
            let query_alignment = aligner.align(&query_buffer, reference);
            let py_query_alignmnet = if with_label {
                let labeled_query_alignment = reference.label_query_alignment(query_alignment);
                PyQueryAlignment::from(labeled_query_alignment)
            } else {
                PyQueryAlignment::from(query_alignment)
            };
            let py_read_alignment = PyReadAlignment{
                read: label_buffer.clone(),
                is_forward: false,
                result: py_query_alignmnet,
            };
            py_read_alignments.push(py_read_alignment);
        }
    }
    PyFastaAlignment(py_read_alignments)
}
#[inline]
fn align_fasta_with_core_aligner_checking_signals<A: Algorithm, R: Read>(
    aligner: &mut Aligner<A>,
    fasta_reader: &mut FastaReader<R>,
    reference: &Reference,
    with_label: bool,
    with_reverse_complementary: bool,
) -> PyResult<PyFastaAlignment> {
    Python::with_gil(|py| -> PyResult<PyFastaAlignment> {
        let mut py_read_alignments = Vec::new();

        let mut query_buffer = Vec::new();
        let mut label_buffer = String::new();
        while let Some(mut record) = fasta_reader.next() {
            query_buffer.clear();
            label_buffer.clear();
            record.extend_seq_buf(&mut query_buffer);
            record.extend_id_string(&mut label_buffer);

            let query_alignment = aligner.align(&query_buffer, reference);
            let py_query_alignmnet = if with_label {
                let labeled_query_alignment = reference.label_query_alignment(query_alignment);
                PyQueryAlignment::from(labeled_query_alignment)
            } else {
                PyQueryAlignment::from(query_alignment)
            };
            let py_read_alignment = PyReadAlignment{
                read: label_buffer.clone(),
                is_forward: true,
                result: py_query_alignmnet,
            };
            py_read_alignments.push(py_read_alignment);

            if with_reverse_complementary {
                query_buffer.reverse();
                let query_alignment = aligner.align(&query_buffer, reference);
                let py_query_alignmnet = if with_label {
                    let labeled_query_alignment = reference.label_query_alignment(query_alignment);
                    PyQueryAlignment::from(labeled_query_alignment)
                } else {
                    PyQueryAlignment::from(query_alignment)
                };
                let py_read_alignment = PyReadAlignment{
                    read: label_buffer.clone(),
                    is_forward: false,
                    result: py_query_alignmnet,
                };
                py_read_alignments.push(py_read_alignment);
            }
            py.check_signals()?;
        }
        Ok(PyFastaAlignment(py_read_alignments))
    })
}
