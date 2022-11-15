use pyo3::prelude::*;
use pyo3::exceptions::PyException;

use super::{
    Reference,
    QueryResult,
    FastaResult,
};

use sigalign::{
    Reference as SigReference,
    Aligner as SigAligner,
    sequence_storage::{
        InMemoryStorage as SigInMemoryStorage,
    },
};

#[pyclass]
pub struct Aligner {
    inner: SigAligner
}

#[pymethods]
impl Aligner {
    #[new]
    #[args(
        is_local_mode = "true",
    )]
    fn new(
        mismatch_penalty: usize,
        gap_open_penalty: usize,
        gap_extend_penalty: usize,
        minimum_aligned_length: usize,
        maximum_penalty_per_length: f32,
        is_local_mode: bool,
    ) -> PyResult<Self> {
        let sig_aligner = if is_local_mode {
            SigAligner::new_local(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        } else {
            SigAligner::new_semi_global(mismatch_penalty, gap_open_penalty, gap_extend_penalty, minimum_aligned_length, maximum_penalty_per_length)
        };
        match sig_aligner {
            Ok(sig_aligner) => Ok(Self { inner: sig_aligner }),
            Err(err) => Err(PyException::new_err(format!("{}", err))),
        }
    }

    // Get options of aligner
    #[getter(penalties)]
    fn get_penalties(&self) -> [usize; 3] {
        self.inner.get_penalties()
    }
    #[getter(cutoffs)]
    fn get_similarity_cutoff(&self) -> (usize, f32) {
        self.inner.get_similarity_cutoff()
    }
    #[getter(pattern_size)]
    fn get_pattern_size(&self) -> usize {
        self.inner.get_pattern_size()
    }
    #[getter(is_local_mode)]
    fn is_local_mode(&self) -> bool {
        self.inner.is_local_mode()
    }

    // Alignments
    fn align_query(
        &mut self,
        reference: &mut Reference,
        query: &str
    ) -> PyResult<QueryResult> {
        let query_bytes = query.as_bytes();
        let result = self.inner.query_labeled_alignment(&reference.inner, query_bytes);

        match result {
            Ok(v) => {
                Ok(QueryResult { inner: v })
            },
            Err(e) => {
                Err(PyException::new_err(format!("{}", e)))
            },
        }
    }
    fn align_fasta_str(
        &mut self,
        reference: &mut Reference,
        fasta_str: &str,
    ) -> PyResult<FastaResult> {
        let fasta_bytes = fasta_str.as_bytes();
        let result = self.inner.fasta_bytes_labeled_alignment(&reference.inner, fasta_bytes);

        Ok(FastaResult { inner: result })
    }
    fn align_fasta_file(
        &mut self,
        reference: &mut Reference,
        fasta_file: &str,
    ) -> PyResult<FastaResult> {
        
        let result = self.inner.fasta_file_labeled_alignment(&reference.inner, fasta_file);

        match result {
            Ok(v) => {
                Ok(FastaResult { inner: v })
            },
            Err(e) => {
                Err(PyException::new_err(format!("{}", e)))
            },
        }
    }
}
