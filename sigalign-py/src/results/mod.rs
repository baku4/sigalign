use pyo3::prelude::*;
use pyo3::exceptions::PyValueError;
use serde::{Deserialize, Serialize};
use serde_json::{
    to_string,
    to_string_pretty,
};

use sigalign::results::{
    AlignmentResult,
    AnchorAlignmentResult,
    AlignmentOperation,
    labeled::{
        LabeledAlignmentResult,
    },
    fasta::{
        FastaAlignmentResult,
        LabeledFastaAlignmentResult,
    },
};

mod py_debug;
mod to_flat_result;
use to_flat_result::{FlatReadResult, FlatTargetResult};

#[pyclass(sequence)]
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct FastaResult(pub Vec<PyReadResult>);
#[pymethods]
impl FastaResult {
    #[new]
    fn py_new(read_results: Vec<PyReadResult>) -> Self {
        Self(read_results)
    }
    fn to_list(&self) -> PyResult<Vec<PyReadResult>> {
        Ok(self.0.clone())
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_table(&self) -> Vec<FlatReadResult> {
        self.to_flat_results()
    }
    fn num_alignments(&self) -> usize {
        self.0.iter().map(|v| v.num_alignments()).sum()
    }
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.len())
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<FastaResultIter>> {
        let iter = FastaResultIter {
            inner: slf.0.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

#[pyclass]
struct FastaResultIter {
    inner: std::vec::IntoIter<PyReadResult>,
}
#[pymethods]
impl FastaResultIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyReadResult> {
        slf.inner.next()
    }
}

#[pyclass(name = "ReadResult")]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PyReadResult {
    read: String,
    result: QueryResult,
}
#[pymethods]
impl PyReadResult {
    #[new]
    fn py_new(read: String, result: QueryResult) -> Self {
        Self {
            read,
            result,
        }
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_table(&self) -> Vec<FlatReadResult> {
        self.to_flat_results()
    }
    pub fn num_alignments(&self) -> usize {
        self.result.num_alignments()
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

#[pyclass(sequence)]
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct QueryResult(pub Vec<PyTargetResult>);
#[pymethods]
impl QueryResult {
    #[new]
    fn py_new(target_results: Vec<PyTargetResult>) -> Self {
        Self(target_results)
    }
    fn to_list(&self) -> PyResult<Vec<PyTargetResult>> {
        Ok(self.0.clone())
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_table(&self) -> Vec<FlatTargetResult> {
        self.to_flat_results()
    }
    pub fn num_alignments(&self) -> usize {
        self.0.iter().map(|v| v.num_alignments()).sum()
    }
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.len())
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<QueryResultIter>> {
        let iter = QueryResultIter {
            inner: slf.0.clone().into_iter(),
        };
        Py::new(slf.py(), iter)
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}
#[pyclass]
struct QueryResultIter {
    inner: std::vec::IntoIter<PyTargetResult>,
}
#[pymethods]
impl QueryResultIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTargetResult> {
        slf.inner.next()
    }
}

#[pyclass(name = "TargetResult")]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PyTargetResult {
    #[pyo3(get)]
    pub index: u32,
    #[pyo3(get)]
    pub label: Option<String>,
    #[pyo3(get)]
    pub alignments: Vec<PyAlignment>,
}
#[pymethods]
impl PyTargetResult {
    #[new]
    fn py_new(index: u32, alignments: Vec<PyAlignment>, label: Option<String>) -> Self {
        PyTargetResult {
            index,
            label,
            alignments: alignments,
        }
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_table(&self) -> Vec<FlatTargetResult> {
        self.to_flat_results()
    }
    pub fn num_alignments(&self) -> usize {
        self.alignments.len()
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

#[pyclass(name = "Alignment")]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PyAlignment {
    #[pyo3(get)]
    pub penalty: u32,
    #[pyo3(get)]
    pub length: u32,
    #[pyo3(get)]
    pub query_position: (u32, u32),
    #[pyo3(get)]
    pub target_position: (u32, u32),
    #[pyo3(get)]
    pub operations: Vec<PyOperation>,
}
#[pymethods]
impl PyAlignment {
    #[new]
    fn new(penalty: u32, length: u32, query_position: (u32, u32), target_position: (u32, u32), operations: Vec<PyOperation>) -> Self {
        PyAlignment {
            penalty,
            length,
            query_position,
            target_position,
            operations,
        }
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

#[pyclass(name = "Operation")]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
#[derive(Serialize, Deserialize)]
pub struct PyOperation {
    #[pyo3(get)]
    pub case: String,
    #[pyo3(get)]
    pub count: u32,
}
#[pymethods]
impl PyOperation {
    #[new]
    fn py_new(case: String, count: u32) -> PyResult<Self> {
        match case.as_str() {
            "M" | "S" | "D" | "I" => {
                Ok(PyOperation {
                    case,
                    count,
                })
            },
            _ => Err(PyValueError::new_err(
                format!("Invalid case argument: {}. Expected one of 'M', 'S', 'D', 'I'.", case)
            ))
        }
        
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

impl FastaResult {
    pub fn from_labeled_fasta_result(
        labeled_fasta_alignment_result: LabeledFastaAlignmentResult
    ) -> Self {
        Self(labeled_fasta_alignment_result.0.into_iter().map(|x| {
            PyReadResult {
                read: x.read,
                result: QueryResult::from_labeled_alignment_result(x.result),
            }
        }).collect())
    }
    pub fn from_fasta_result(
        fasta_alignment_result: FastaAlignmentResult
    ) -> Self {
        Self(fasta_alignment_result.0.into_iter().map(|x| {
            PyReadResult {
                read: x.read,
                result: QueryResult::from_alignment_result(x.result),
            }
        }).collect())
    }
}

impl QueryResult {
    pub fn from_labeled_alignment_result(alignment_result: LabeledAlignmentResult) -> Self {
        Self(alignment_result.0.into_iter().map(|x| {
            PyTargetResult {
                index: x.index,
                label: Some(x.label),
                alignments: x.alignments.into_iter().map(|v| {
                    PyAlignment::from_anchor_alignment_result(v)
                }).collect(),
            }
        }).collect())
    }
    pub fn from_alignment_result(alignment_result: AlignmentResult) -> Self {
        Self(alignment_result.0.into_iter().map(|x| {
            PyTargetResult {
                index: x.index,
                label: None,
                alignments: x.alignments.into_iter().map(|v| {
                    PyAlignment::from_anchor_alignment_result(v)
                }).collect(),
            }
        }).collect())
    }
}

impl PyAlignment {
    fn from_anchor_alignment_result(anchor_alignment_result :AnchorAlignmentResult) -> Self {
        Self {
            penalty: anchor_alignment_result.penalty,
            length: anchor_alignment_result.length,
            query_position: anchor_alignment_result.position.query,
            target_position: anchor_alignment_result.position.target,
            operations: anchor_alignment_result.operations.into_iter().map(|v| {
                PyOperation {
                    case: match v.operation {
                        AlignmentOperation::Match => "M".to_string(),
                        AlignmentOperation::Subst => "S".to_string(),
                        AlignmentOperation::Insertion => "I".to_string(),
                        AlignmentOperation::Deletion => "D".to_string(),
                    },
                    count: v.count,
                }
            }).collect(),
        }
    }
}

pub fn get_result_module(py: Python) -> PyResult<&PyModule> {
    let results_module = PyModule::new(py, "results")?;
    results_module.add_class::<FastaResult>()?;
    results_module.add_class::<PyReadResult>()?;
    results_module.add_class::<QueryResult>()?;
    results_module.add_class::<PyTargetResult>()?;
    results_module.add_class::<PyAlignment>()?;
    results_module.add_class::<PyOperation>()?;
    py.import("sys")?
        .getattr("modules")?
        .set_item("sigalign.results", results_module)?;
    Ok(results_module)
}
