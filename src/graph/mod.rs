
pub mod networkx_graph;

use pyo3::prelude::*;

#[pymodule]
pub fn graph(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<networkx_graph::NetworkXGraph>()?;
    Ok(())
}