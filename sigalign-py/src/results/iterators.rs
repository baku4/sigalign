use pyo3::prelude::*;

use super::{PyReadAlignment, PyTargetAlignment};

#[pyclass]
pub struct FastaAlignmentIter {
    inner: std::vec::IntoIter<PyReadAlignment>,
}
#[pymethods]
impl FastaAlignmentIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyReadAlignment> {
        slf.inner.next()
    }
}
impl FastaAlignmentIter {
    pub fn new(inner: Vec<PyReadAlignment>) -> Self {
        Self {
            inner: inner.into_iter(),
        }
    }
}

#[pyclass]
pub struct QueryAlignmentIter {
    inner: std::vec::IntoIter<PyTargetAlignment>,
}
#[pymethods]
impl QueryAlignmentIter {
    fn __iter__(slf: PyRef<'_, Self>) -> PyRef<'_, Self> {
        slf
    }
    fn __next__(mut slf: PyRefMut<'_, Self>) -> Option<PyTargetAlignment> {
        slf.inner.next()
    }
}
impl QueryAlignmentIter {
    pub fn new(inner: Vec<PyTargetAlignment>) -> Self {
        Self {
            inner: inner.into_iter(),
        }
    }
}
