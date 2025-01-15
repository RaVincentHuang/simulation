
pub mod networkx_graph;


use crate::utils::register_child_module;
use pyo3::prelude::*;

#[pymodule]
pub fn graph(_py: Python, m: &PyModule) -> PyResult<()> {
    register_child_module(_py, m, networkx_graph::networkx_graph, "networkx_graph")?;
    Ok(())
}