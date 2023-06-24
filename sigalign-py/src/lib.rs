use pyo3::prelude::*;

mod reference;
mod results;
mod aligner;

use reference::PyReference;
use aligner::PyAligner;
use results::get_result_module;

#[pymodule]
fn sigalign(py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyReference>()?;
    m.add_class::<PyAligner>()?;
    let results_module = get_result_module(py)?;
    m.add_submodule(results_module)?;
    Ok(())
}
