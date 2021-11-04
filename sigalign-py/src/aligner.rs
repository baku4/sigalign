use crate::{Result, error_msg};
use crate::reference::SigReferenceHolder;
use super::reference::Reference;

use sigalign::Aligner as SigAligner;

use pyo3::prelude::*;
use pyo3::exceptions::PyException;

#[pyclass]
pub struct Aligner {
    sig_aligner: SigAligner
}

#[pymethods]
impl Aligner {
    #[new]
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
    ) -> PyResult<Self> {
        let sig_aligner = SigAligner::new(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length);

        match sig_aligner {
            Ok(sig_aligner) => Ok(Self { sig_aligner }),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
    
    fn semi_global_alignment(
        &self,
        reference: &mut Reference,
        query: &str
    ) -> PyResult<String> {
        let query_bytes = query.as_bytes();
        
        let result = reference.sig_reference_holder.semi_global_alignment(
            &self.sig_aligner,
            query_bytes
        );

        match result {
            Ok(json) => Ok(json),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
    fn semi_global_alignment_labeled(
        &self,
        reference: &mut Reference,
        query: &str
    ) -> PyResult<String> {
        let query_bytes = query.as_bytes();
        
        let result = reference.sig_reference_holder.semi_global_alignment_labeled(
            &self.sig_aligner,
            query_bytes
        );

        match result {
            Ok(json) => Ok(json),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }

    fn local_alignment(
        &self,
        reference: &mut Reference,
        query: &str
    ) -> PyResult<String> {
        let query_bytes = query.as_bytes();
        
        let result = reference.sig_reference_holder.local_alignment(
            &self.sig_aligner,
            query_bytes
        );

        match result {
            Ok(json) => Ok(json),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
    fn local_alignment_labeled(
        &self,
        reference: &mut Reference,
        query: &str
    ) -> PyResult<String> {
        let query_bytes = query.as_bytes();
        
        let result = reference.sig_reference_holder.local_alignment_labeled(
            &self.sig_aligner,
            query_bytes
        );

        match result {
            Ok(json) => Ok(json),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }
}

impl SigReferenceHolder {
    fn semi_global_alignment(
        &mut self,
        sig_aligner: &SigAligner,
        query: &[u8],
    ) -> Result<String> {
        match self {
            Self::InMemory(sig_reference) => {
                sig_aligner.semi_global_alignment(sig_reference, query)
            },
            Self::IndexedFasta(sig_reference) => {
                sig_aligner.semi_global_alignment(sig_reference, query)
            },
        }
    }
    fn semi_global_alignment_labeled(
        &mut self,
        sig_aligner: &SigAligner,
        query: &[u8],
    ) -> Result<String> {
        match self {
            Self::InMemory(sig_reference) => {
                sig_aligner.semi_global_alignment_labeled(sig_reference, query)
            },
            Self::IndexedFasta(sig_reference) => {
                sig_aligner.semi_global_alignment_labeled(sig_reference, query)
            },
        }
    }
    fn local_alignment(
        &mut self,
        sig_aligner: &SigAligner,
        query: &[u8],
    ) -> Result<String> {
        match self {
            Self::InMemory(sig_reference) => {
                sig_aligner.local_alignment(sig_reference, query)
            },
            Self::IndexedFasta(sig_reference) => {
                sig_aligner.local_alignment(sig_reference, query)
            },
        }
    }
    fn local_alignment_labeled(
        &mut self,
        sig_aligner: &SigAligner,
        query: &[u8],
    ) -> Result<String> {
        match self {
            Self::InMemory(sig_reference) => {
                sig_aligner.local_alignment_labeled(sig_reference, query)
            },
            Self::IndexedFasta(sig_reference) => {
                sig_aligner.local_alignment_labeled(sig_reference, query)
            },
        }
    }
}