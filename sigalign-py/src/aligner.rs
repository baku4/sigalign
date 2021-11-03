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
    #[pyo3(text_signature = "($self)")]
    fn semi_global_alignment(
        &self,
        reference: &mut Reference,
        query: String
    ) {// -> String {
        let query_bytes = query.as_bytes();

        // self.sig_aligner.semi_global_alignment(query).unwrap()
    }
}