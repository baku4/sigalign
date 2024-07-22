use pyo3::prelude::*;

mod reference;
// mod aligner;
// mod results;

use reference::PyReference;
// use aligner::PyAligner;
// use results::get_result_module;

#[pymodule]
fn sigalign(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReference>()?;
    // m.add_class::<PyAligner>()?;
    // let results_module = get_result_module(py)?;
    // m.add_submodule(results_module)?;
    Ok(())
}
