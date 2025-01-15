pub mod graph;
pub mod utils;

use pyo3::prelude::*;
use utils::register_child_module;

// pub fn add(left: u64, right: u64) -> u64 {
//     left + right
// }



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        // let result = add(2, 2);
        // assert_eq!(result, 4);
    }
}

#[pymodule]
fn simulation(_py: Python, m: &PyModule) -> PyResult<()> {
    register_child_module(_py, m, graph::graph, "graph")?;
    Ok(())
}
