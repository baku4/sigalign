use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyString};

use crate::{
    reference::PyReference,
    results::{PyFastaAlignment, PyQueryAlignment},
};

mod wrapper_for_algorithm;
use wrapper_for_algorithm::AlignerWrapper;

#[pyclass(name = "Aligner")]
pub struct PyAligner {
    pub aligner_wrapper: AlignerWrapper,
    pub limitation_holder: Option<u32>,
    pub chunk_holder: Option<(u32, u32)>,
}

#[pymethods]
impl PyAligner {
    #[new]
    #[pyo3(signature = (
        px,
        po,
        pe,
        minl,
        maxp,
        use_local_mode=true,
        use_limit=None,
        use_chunk=None,
    ))]
    fn py_new(
        px: u32,
        po: u32,
        pe: u32,
        minl: u32,
        maxp: f32,
        use_local_mode: bool,
        use_limit: Option<u32>,
        use_chunk: Option<(u32, u32)>,
    ) -> PyResult<Self> {
        let aligner_wrapper =
            AlignerWrapper::new(px, po, pe, minl, maxp, use_local_mode, use_limit, use_chunk)?;
        Ok(PyAligner {
            aligner_wrapper,
            limitation_holder: use_limit,
            chunk_holder: use_chunk,
        })
    }

    #[getter(px)]
    fn get_mismatch_penalty(&self) -> u32 {
        self.aligner_wrapper.get_mismatch_penalty()
    }
    #[getter(po)]
    fn get_gap_open_penalty(&self) -> u32 {
        self.aligner_wrapper.get_gap_open_penalty()
    }
    #[getter(pe)]
    fn get_gap_extend_penalty(&self) -> u32 {
        self.aligner_wrapper.get_gap_extend_penalty()
    }
    #[getter(minl)]
    fn get_minimum_length(&self) -> u32 {
        self.aligner_wrapper.get_minimum_length()
    }
    #[getter(maxp)]
    fn get_maximum_penalty_per_length(&self) -> f32 {
        self.aligner_wrapper.get_maximum_penalty_per_length()
    }
    #[getter(pattern_size)]
    fn get_pattern_size(&self) -> u32 {
        self.aligner_wrapper.get_pattern_size()
    }
    #[getter(algorithm)]
    fn get_algorithm(&self) -> &str {
        self.aligner_wrapper.as_str()
    }
    #[getter(is_local_mode)]
    fn is_local_mode(&self) -> bool {
        self.aligner_wrapper.is_local_mode()
    }
    #[getter(limitation)]
    fn get_limitation(&self) -> Option<u32> {
        self.limitation_holder
    }
    #[getter(chunk)]
    fn get_chunk(&self) -> Option<(u32, u32)> {
        self.chunk_holder
    }

    // Alignments
    #[pyo3(signature = (
        query,
        reference,
        with_label=false,
    ))]
    fn align_query(
        &mut self,
        query: &Bound<PyAny>,
        reference: &PyReference,
        with_label: bool,
    ) -> PyResult<PyQueryAlignment> {
        let query_bytes = if query.is_instance_of::<PyString>() {
            query.downcast::<PyString>()?.to_str()?.as_bytes()
        } else if query.is_instance_of::<PyBytes>() {
            query.downcast::<PyBytes>()?.as_bytes()
        } else {
            return Err(PyValueError::new_err(
                "The input must be either a string or bytes.",
            ));
        };

        let py_query_alignment =
            self.aligner_wrapper
                .align_query(query_bytes, reference, with_label);
        Ok(py_query_alignment)
    }
    #[pyo3(signature = (
        file_path,
        reference,
        with_label=false,
        with_reverse_complementary=false,
        allow_interrupt=false,
    ))]
    fn align_fasta_file(
        &mut self,
        file_path: &str,
        reference: &PyReference,
        with_label: bool,
        with_reverse_complementary: bool,
        allow_interrupt: bool,
    ) -> PyResult<PyFastaAlignment> {
        self.aligner_wrapper.align_fasta_file(
            reference,
            file_path,
            with_label,
            with_reverse_complementary,
            allow_interrupt,
        )
    }
    #[pyo3(signature = (
        fasta,
        reference,
        with_label=false,
        with_reverse_complementary=false,
        allow_interrupt=false,
    ))]
    fn align_fasta(
        &mut self,
        fasta: &Bound<PyAny>,
        reference: &PyReference,
        with_label: bool,
        with_reverse_complementary: bool,
        allow_interrupt: bool,
    ) -> PyResult<PyFastaAlignment> {
        let fasta_bytes = if fasta.is_instance_of::<PyString>() {
            fasta.downcast::<PyString>()?.to_str()?.as_bytes()
        } else if fasta.is_instance_of::<PyBytes>() {
            fasta.downcast::<PyBytes>()?.as_bytes()
        } else {
            return Err(PyValueError::new_err(
                "The input must be either a string or bytes.",
            ));
        };

        self.aligner_wrapper.align_fasta_bytes(
            reference,
            fasta_bytes,
            with_label,
            with_reverse_complementary,
            allow_interrupt,
        )
    }
    #[pyo3(signature = (
        file_path,
        reference,
        with_label=false,
        with_reverse_complementary=false,
        allow_interrupt=false,
    ))]
    fn align_fastq_file(
        &mut self,
        file_path: &str,
        reference: &PyReference,
        with_label: bool,
        with_reverse_complementary: bool,
        allow_interrupt: bool,
    ) -> PyResult<PyFastaAlignment> {
        self.aligner_wrapper.align_fastq_file(
            reference,
            file_path,
            with_label,
            with_reverse_complementary,
            allow_interrupt,
        )
    }
    #[pyo3(signature = (
        fastq,
        reference,
        with_label=false,
        with_reverse_complementary=false,
        allow_interrupt=false,
    ))]
    fn align_fastq(
        &mut self,
        fastq: &Bound<PyAny>,
        reference: &PyReference,
        with_label: bool,
        with_reverse_complementary: bool,
        allow_interrupt: bool,
    ) -> PyResult<PyFastaAlignment> {
        let fastq_bytes = if fastq.is_instance_of::<PyString>() {
            fastq.downcast::<PyString>()?.to_str()?.as_bytes()
        } else if fastq.is_instance_of::<PyBytes>() {
            fastq.downcast::<PyBytes>()?.as_bytes()
        } else {
            return Err(PyValueError::new_err(
                "The input must be either a string or bytes.",
            ));
        };

        self.aligner_wrapper.align_fastq_bytes(
            reference,
            fastq_bytes,
            with_label,
            with_reverse_complementary,
            allow_interrupt,
        )
    }
}
