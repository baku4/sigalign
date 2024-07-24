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

use crate::{
    reference::PyReference,
    results::{
        PyFastaAlignment,
        PyReadAlignment,
        PyQueryAlignment,
        PyTargetAlignment,
        PyAlignment,
        PyAlignmentOperations,
        PyAlignmentOperation,
    },
};

mod wrapper_for_algorithm;
use wrapper_for_algorithm::AlignerWrapper;

#[pyclass(name = "Aligner")]
pub struct PyAligner {
    pub aligner_wrapper: AlignerWrapper,
    pub limitation_holder: Option<u32>,
    pub chunked_holder: Option<(u32, u32)>,
}

#[pymethods]
impl PyAligner {
    #[new]
    fn py_new(
        px: u32,
        po: u32,
        pe: u32,
        minl: u32,
        maxp: f32,
        use_local_mode: bool,
        use_limit: Option<u32>,
        use_chunked_mode: Option<(u32, u32)>,
    ) -> PyResult<Self> {
        todo!()
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
    #[getter(limitation)]
    fn get_limitation(&self) -> Option<u32> {
        self.limitation_holder
    }

    // Alignments
    #[pyo3(signature = (
        reference,
        query,
        with_label=false,
    ))]
    fn align_query(
        &mut self,
        reference: &PyReference,
        query: &str,
        with_label: bool,
    ) -> PyResult<PyQueryAlignment> {
        let res =  self.aligner_wrapper.align_query(query, reference);
        let py_query_alignment = if with_label {
            let sig_reference = reference.as_ref();
            let labeled_res = sig_reference.label_query_alignment(res);
            PyQueryAlignment::from(labeled_res)
        } else {
            PyQueryAlignment::from(res)
        };
        Ok(py_query_alignment)
    }
    // #[pyo3(signature = (
    //     reference,
    //     fasta_file,
    //     with_label=true,
    //     accept_interrupt=true,
    // ))]
    // fn align_fasta_file(
    //     &mut self,
    //     reference: &PyReference,
    //     fasta_file: &str,
    //     with_label: bool,
    //     accept_interrupt: bool,
    // ) -> PyResult<FastaResult> {
    //     let result = if with_label {
    //         let labeled_result = if accept_interrupt {
    //             self.align_fasta_file_with_label_checking_signals(reference, fasta_file)?
    //         } else {
    //             self.aligner.align_fasta_file_labeled(&reference.reference, fasta_file)
    //                 .map_err(convert_error)?
    //         };
    //         FastaResult::from_labeled_fasta_result(labeled_result)
    //     } else {
    //         let unlabeled_result = if accept_interrupt {
    //             self.align_fasta_file_without_label_checking_signals(reference, fasta_file)?
    //         } else {
    //             self.aligner.align_fasta_file(&reference.reference, fasta_file)
    //                 .map_err(convert_error)?
    //         };
    //         FastaResult::from_fasta_result(unlabeled_result)
    //     };
    //     Ok(result)
    // }
}

// fn convert_error(err: AlignmentError) -> PyErr {
//     match err {
//         AlignmentError::IoError(v) => v.into(),
//         _ => {
//             PyException::new_err(
//                 format!("{}", err)
//             )
//         }
//     }
// }

// use sigalign::utils::FastaReader;
// impl PyAligner {
//     fn align_fasta_file_with_label_checking_signals(
//         &mut self,
//         reference: &PyReference,
//         fasta_file: &str,
//     ) -> PyResult<LabeledFastaAlignmentResult> {
//         Python::with_gil(|py| -> PyResult<LabeledFastaAlignmentResult> {
//             let fasta_reader = FastaReader::from_path(fasta_file)?;
//             let mut sequence_buffer = reference.reference.get_sequence_buffer();
//             let mut results = Vec::new();
//             for (label, query) in fasta_reader {
//                 let result = self.aligner.alignment(
//                     &reference.reference,
//                     &mut sequence_buffer,
//                     &query,
//                 ).to_labeled_result_unchecked(&reference.reference);
//                 if result.result_counts() != 0 {
//                     results.push(LabeledReadAlignmentResult {
//                         read: label,
//                         result,
//                     });
//                 }
//                 py.check_signals()?;
//             }
//             Ok(LabeledFastaAlignmentResult(results))
//         })
//     }
//     fn align_fasta_file_without_label_checking_signals(
//         &mut self,
//         reference: &PyReference,
//         fasta_file: &str,
//     ) -> PyResult<FastaAlignmentResult> {
//         Python::with_gil(|py| -> PyResult<FastaAlignmentResult> {
//             let fasta_reader = FastaReader::from_path(fasta_file)?;
//             let mut sequence_buffer = reference.reference.get_sequence_buffer();
//             let mut results = Vec::new();
//             for (label, query) in fasta_reader {
//                 let result = self.aligner.alignment(
//                     &reference.reference,
//                     &mut sequence_buffer,
//                     &query,
//                 );
//                 if result.result_counts() != 0 {
//                     results.push(ReadAlignmentResult {
//                         read: label,
//                         result,
//                     });
//                 }
//                 py.check_signals()?;
//             }
//             Ok(FastaAlignmentResult(results))
//         })
//     }
// }
