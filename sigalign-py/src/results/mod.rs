use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{to_string, to_string_pretty};

mod iterators; // Contains Python's iterators classes.
pub use iterators::{FastaAlignmentIter, QueryAlignmentIter};
mod from;
mod py_debug;
mod to_flat_result;
use to_flat_result::{FlatReadAlignment, FlatTargetAlignment};

pub fn register_results_module_as_submodule(parent_module: &Bound<'_, PyModule>) -> PyResult<()> {
    let results_module = PyModule::new_bound(parent_module.py(), "results")?;
    results_module.add_class::<PyFastaAlignment>()?;
    results_module.add_class::<PyReadAlignment>()?;
    results_module.add_class::<PyQueryAlignment>()?;
    results_module.add_class::<PyTargetAlignment>()?;
    results_module.add_class::<PyAlignment>()?;
    results_module.add_class::<PyAlignmentOperations>()?;
    results_module.add_class::<PyAlignmentOperation>()?;
    parent_module.add_submodule(&results_module)?;
    Ok(())
}

/// Not in Rust library.
#[pyclass(name = "FastaAlignment", sequence, frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PyFastaAlignment(pub Vec<PyReadAlignment>);
#[pymethods]
impl PyFastaAlignment {
    #[new]
    fn py_new(read_alignments: Vec<PyReadAlignment>) -> Self {
        Self(read_alignments)
    }
    fn to_list(&self) -> PyResult<Vec<PyReadAlignment>> {
        Ok(self.0.clone())
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_rows(&self) -> Vec<FlatReadAlignment> {
        self.to_flat_results()
    }
    fn num_alignments(&self) -> usize {
        self.0.iter().map(|v| v.num_alignments()).sum()
    }
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.len())
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<FastaAlignmentIter>> {
        let iter = FastaAlignmentIter::new(slf.0.clone());
        Py::new(slf.py(), iter)
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

/// Not in Rust library.
#[pyclass(name = "ReadAlignment", sequence, frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PyReadAlignment {
    #[pyo3(get)]
    pub read: String,
    #[pyo3(get)]
    pub is_forward: bool,
    #[pyo3(get)]
    pub result: PyQueryAlignment,
}
#[pymethods]
impl PyReadAlignment {
    #[new]
    #[pyo3(signature = (read, result, is_forward = true))]
    fn py_new(read: String, result: PyQueryAlignment, is_forward: bool) -> Self {
        Self {
            read,
            is_forward,
            result,
        }
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_rows(&self) -> Vec<FlatReadAlignment> {
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

/// Represents the both `QueryAlignment` and `LabeledQueryAlignment`.
#[pyclass(name = "QueryAlignment", sequence, frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PyQueryAlignment(pub Vec<PyTargetAlignment>);
#[pymethods]
impl PyQueryAlignment {
    #[new]
    fn py_new(target_alignments: Vec<PyTargetAlignment>) -> Self {
        Self(target_alignments)
    }
    fn to_list(&self) -> PyResult<Vec<PyTargetAlignment>> {
        Ok(self.0.clone())
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_rows(&self) -> Vec<FlatTargetAlignment> {
        self.to_flat_results()
    }
    pub fn num_alignments(&self) -> usize {
        self.0.iter().map(|v| v.num_alignments()).sum()
    }
    fn __len__(&self) -> PyResult<usize> {
        Ok(self.0.len())
    }
    fn __iter__(slf: PyRef<'_, Self>) -> PyResult<Py<QueryAlignmentIter>> {
        let iter = QueryAlignmentIter::new(slf.0.clone());
        Py::new(slf.py(), iter)
    }
    fn __str__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
    fn __repr__(&self) -> PyResult<String> {
        Ok(self.py_debug())
    }
}

/// Represents the both `TargetAlignment` and `LabeledTargetAlignment`.
#[pyclass(name = "TargetAlignment", frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PyTargetAlignment {
    #[pyo3(get)]
    pub index: u32,
    #[pyo3(get)]
    pub label: Option<String>,
    #[pyo3(get)]
    pub alignments: Vec<PyAlignment>,
}
#[pymethods]
impl PyTargetAlignment {
    #[new]
    #[pyo3(signature = (index, alignments, label=None))]
    fn py_new(index: u32, alignments: Vec<PyAlignment>, label: Option<String>) -> Self {
        Self {
            index,
            label,
            alignments,
        }
    }
    fn to_json(&self) -> String {
        to_string(self).unwrap()
    }
    fn to_json_pretty(&self) -> String {
        to_string_pretty(self).unwrap()
    }
    fn to_rows(&self) -> Vec<FlatTargetAlignment> {
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

// Instead of `AlignmentPosition`, use tuple `(u32, u32)` for `query` and `target`, for simplicity.
#[pyclass(name = "Alignment", frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
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
    pub operations: Vec<PyAlignmentOperations>,
}
#[pymethods]
impl PyAlignment {
    #[new]
    fn new(
        penalty: u32,
        length: u32,
        query_position: (u32, u32),
        target_position: (u32, u32),
        operations: Vec<PyAlignmentOperations>,
    ) -> Self {
        Self {
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

#[pyclass(name = "AlignmentOperations", frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub struct PyAlignmentOperations {
    #[pyo3(get)]
    pub operation: PyAlignmentOperation,
    #[pyo3(get)]
    pub count: u32,
}
#[pymethods]
impl PyAlignmentOperations {
    #[new]
    fn py_new(operation: PyAlignmentOperation, count: u32) -> PyResult<Self> {
        Ok(Self { operation, count })
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

#[pyclass(name = "AlignmentOperation", frozen, eq, hash)]
#[derive(Debug, PartialEq, Eq, Hash, Clone, Serialize, Deserialize)]
pub enum PyAlignmentOperation {
    Match,
    Subst,
    Insertion,
    Deletion,
}
#[pymethods]
impl PyAlignmentOperation {
    #[new]
    fn py_new(chr: &str) -> PyResult<Self> {
        match chr {
            "M" | "Match" => Ok(Self::Match),
            "S" | "Subst" => Ok(Self::Subst),
            "I" | "Insertion" => Ok(Self::Insertion),
            "D" | "Deletion" => Ok(Self::Deletion),
            _ => Err(PyValueError::new_err(
                format!("Invalid alignment operation: {}. Expected one of 'M (or Match)', 'S (or Subst)', 'I (or Insertion)', 'D (or Deletion)'.", chr)
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
