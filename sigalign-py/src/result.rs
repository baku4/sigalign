use pyo3::prelude::*;
use pyo3::exceptions::PyException;

use sigalign::{
    result::{
        AlignmentLabeledResult,
        FastaAlignmentLabeledResult,
    }
};

#[pyclass]
pub struct QueryResult {
    pub inner: AlignmentLabeledResult,
}
#[pymethods]
impl QueryResult {
    pub fn to_json(&self) -> String {
        self.inner.to_json()
    }
    pub fn to_2d_array(&self) -> Vec<Query2DArray> {
        let result_count: usize = self.inner.0.iter()
            .map(|v| v.alignments.len())
            .sum();
        let mut array_result = Vec::with_capacity(result_count);
        self.inner.0.iter().for_each(|record_result| {
            record_result.alignments.iter().for_each(|alignment| {
                array_result.push((
                    record_result.index, record_result.label.clone(),
                    alignment.penalty, alignment.length,
                    alignment.position.query.0, alignment.position.query.1,
                    alignment.position.record.0, alignment.position.record.1,
                    operations_to_string(&alignment.operations),
                ))
            });
        });
        array_result
    }
}
type Query2DArray = (
    usize, // index of record
    String, // label of record
    usize, // penalty
    usize, // length
    usize, // query start index
    usize, // query end index
    usize, // record start index
    usize, // record end index
    String, // operations
);

#[pyclass]
pub struct FastaResult {
    pub inner: FastaAlignmentLabeledResult,
}
#[pymethods]
impl FastaResult {
    pub fn to_json(&self) -> String {
        self.inner.to_json()
    }
    pub fn to_2d_array(&self) -> Vec<Fasta2DArray> {
        let result_count: usize = self.inner.0.iter()
            .map(|v| v.result_counts())
            .sum();
        let mut array_result = Vec::with_capacity(result_count);
        self.inner.0.iter().for_each(|read_result| {
            read_result.result.0.iter().for_each(|record_result| {
                record_result.alignments.iter().for_each(|alignment| {
                    array_result.push((
                        read_result.read.clone(), record_result.index, record_result.label.clone(),
                        alignment.penalty, alignment.length,
                        alignment.position.query.0, alignment.position.query.1,
                        alignment.position.record.0, alignment.position.record.1,
                        operations_to_string(&alignment.operations),
                    ))
                });
            });
        });
        array_result
    }
}
type Fasta2DArray = (
    String, // query name
    usize, // index of record
    String, // label of record
    usize, // penalty
    usize, // length
    usize, // query start index
    usize, // query end index
    usize, // record start index
    usize, // record end index
    String, // operations
);

use sigalign::result::{AlignmentOperation, AlignmentCase};
fn operations_to_string(operations: &Vec<AlignmentOperation>) -> String {
    let string_ops: Vec<String> = operations.iter().map(|op| {
        format!(
            "{}{}",
            match op.case {
                AlignmentCase::Match => 'M',
                AlignmentCase::Subst => 'S',
                AlignmentCase::Insertion => 'I',
                AlignmentCase::Deletion => 'D',
            },
            op.count,
        )
    }).collect();
    string_ops.concat()
}

pub fn register_result_module(py: Python<'_>, parent_module: &PyModule) -> PyResult<()> {
    let result_module = PyModule::new(py, "result")?;
    result_module.add_class::<QueryResult>()?;
    result_module.add_class::<FastaResult>()?;
    parent_module.add_submodule(result_module)?;
    Ok(())
}
