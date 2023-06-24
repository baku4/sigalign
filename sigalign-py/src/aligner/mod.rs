use pyo3::prelude::*;
use pyo3::exceptions::{
    PyException,
    PyValueError,
};
use sigalign::results::{
    fasta::{
        FastaAlignmentResult,
        ReadAlignmentResult,
        LabeledFastaAlignmentResult,
        LabeledReadAlignmentResult,
    },
};

use crate::{
    reference::PyReference,
    results::*,
};

use sigalign::wrapper::DefaultAligner;
use sigalign::aligner::AlignmentError;
#[pyclass(name = "Aligner")]
#[repr(transparent)]
pub struct PyAligner {
    pub aligner: DefaultAligner
}

#[pymethods]
impl PyAligner {
    #[new]
    #[pyo3(signature = (
        px,
        po,
        pe,
        ml,
        mpl,
        local_mode=true,
    ))]
    fn py_new(
        px: u32,
        po: u32,
        pe: u32,
        ml: u32,
        mpl: f32,
        local_mode: bool,
    ) -> PyResult<Self> {
        let aligner = if local_mode {
            DefaultAligner::new_local(px, po, pe, ml, mpl)
        } else {
            DefaultAligner::new_semi_global(px, po, pe, ml, mpl)
        };
        match aligner {
            Ok(aligner) => Ok(Self { aligner }),
            Err(err) => Err(PyValueError::new_err(format!("{}", err))),
        }
    }

    #[getter(px)]
    fn get_mismatch_penalty(&self) -> u32 {
        self.aligner.get_mismatch_penalty()
    }
    #[getter(po)]
    fn get_gap_open_penalty(&self) -> u32 {
        self.aligner.get_gap_open_penalty()
    }
    #[getter(pe)]
    fn get_gap_extend_penalty(&self) -> u32 {
        self.aligner.get_gap_extend_penalty()
    }
    #[getter(ml)]
    fn get_minimum_aligned_length(&self) -> u32 {
        self.aligner.get_minimum_aligned_length()
    }
    #[getter(mpl)]
    fn get_maximum_penalty_per_length(&self) -> f32 {
        self.aligner.get_maximum_penalty_per_length()
    }
    #[getter(pattern_size)]
    fn get_pattern_size(&self) -> u32 {
        self.aligner.get_pattern_size()
    }
    #[getter(is_local_mode)]
    fn is_local_mode(&self) -> bool {
        self.aligner.is_local_mode()
    }

    // Alignments
    #[pyo3(signature = (
        reference,
        query,
        with_label=true,
    ))]
    fn align_query(
        &mut self,
        reference: &PyReference,
        query: &str,
        with_label: bool,
    ) -> PyResult<QueryResult> {
        let query_bytes = query.as_bytes();
        let query_result = if with_label {
            let labeled_result = self.aligner.align_query_labeled(
                &reference.reference,
                query_bytes,
            );
            QueryResult::from_labeled_alignment_result(labeled_result)
        } else {
            let result = self.aligner.align_query(
                &reference.reference,
                query_bytes,
            );
            QueryResult::from_alignment_result(result)
        };

        Ok(query_result)
    }
    #[pyo3(signature = (
        reference,
        fasta_file,
        with_label=true,
        accept_interrupt=true,
    ))]
    fn align_fasta_file(
        &mut self,
        reference: &PyReference,
        fasta_file: &str,
        with_label: bool,
        accept_interrupt: bool,
    ) -> PyResult<FastaResult> {
        let result = if with_label {
            let labeled_result = if accept_interrupt {
                self.align_fasta_file_with_label_checking_signals(reference, fasta_file)?
            } else {
                self.aligner.align_fasta_file_labeled(&reference.reference, fasta_file)
                    .map_err(convert_error)?
            };
            FastaResult::from_labeled_fasta_result(labeled_result)
        } else {
            let unlabeled_result = if accept_interrupt {
                self.align_fasta_file_without_label_checking_signals(reference, fasta_file)?
            } else {
                self.aligner.align_fasta_file(&reference.reference, fasta_file)
                    .map_err(convert_error)?
            };
            FastaResult::from_fasta_result(unlabeled_result)
        };
        Ok(result)
    }
}

fn convert_error(err: AlignmentError) -> PyErr {
    match err {
        AlignmentError::IoError(v) => v.into(),
        _ => {
            PyException::new_err(
                format!("{}", err)
            )
        }
    }
}

use sigalign::utils::FastaReader;
impl PyAligner {
    fn align_fasta_file_with_label_checking_signals(
        &mut self,
        reference: &PyReference,
        fasta_file: &str,
    ) -> PyResult<LabeledFastaAlignmentResult> {
        Python::with_gil(|py| -> PyResult<LabeledFastaAlignmentResult> {
            let fasta_reader = FastaReader::from_path(fasta_file)?;
            let mut sequence_buffer = reference.reference.get_sequence_buffer();
            let mut results = Vec::new();
            for (label, query) in fasta_reader {
                let result = self.aligner.alignment(
                    &reference.reference,
                    &mut sequence_buffer,
                    &query,
                ).to_labeled_result_unchecked(&reference.reference);
                if result.result_counts() != 0 {
                    results.push(LabeledReadAlignmentResult {
                        read: label,
                        result,
                    });
                }
                py.check_signals()?;
            }
            Ok(LabeledFastaAlignmentResult(results))
        })
    }
    fn align_fasta_file_without_label_checking_signals(
        &mut self,
        reference: &PyReference,
        fasta_file: &str,
    ) -> PyResult<FastaAlignmentResult> {
        Python::with_gil(|py| -> PyResult<FastaAlignmentResult> {
            let fasta_reader = FastaReader::from_path(fasta_file)?;
            let mut sequence_buffer = reference.reference.get_sequence_buffer();
            let mut results = Vec::new();
            for (label, query) in fasta_reader {
                let result = self.aligner.alignment(
                    &reference.reference,
                    &mut sequence_buffer,
                    &query,
                );
                if result.result_counts() != 0 {
                    results.push(ReadAlignmentResult {
                        read: label,
                        result,
                    });
                }
                py.check_signals()?;
            }
            Ok(FastaAlignmentResult(results))
        })
    }
}
