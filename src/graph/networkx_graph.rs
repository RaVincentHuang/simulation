use graph_simulation::algorithm::simulation::Simulation;
use graph_simulation::graph::base::Graph;
use pyo3::types::PySet;
use pyo3::{prelude::*, types::PyDict};
use graph_simulation::graph::labeled_graph::{Label, Labeled};
use graph_simulation::graph::base::{Directed, Adjacency, AdjacencyInv};


use pyo3::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::path::Display;
use std::sync::Arc;

type SharedRustFn = Arc<dyn Fn(&Attributes, &Attributes) -> bool + Send + Sync>;


// 自定义图结构

#[derive(Debug)]
struct Attributes(HashMap<String, Py<PyAny>>);

impl Clone for Attributes {
    fn clone(&self) -> Self {
        Python::with_gil(|py| {
            let cloned_map = self.0.iter()
                .map(|(k, v)| (k.clone(), v.clone_ref(py)))
                .collect();
            Attributes(cloned_map)
        })
    }
}

impl std::fmt::Display for Attributes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        write!(f, "{{")?;
        for (key, value) in entries {
            write!(f, "{}: {}, ", key, value)?;
        }
        write!(f, "}}")
    }
}

impl PartialEq for Attributes {
    fn eq(&self, other: &Self) -> bool {
        // 首先比较长度
        if self.0.len() != other.0.len() {
            return false;
        }

        Python::with_gil(|py| {
            self.0.iter().all(|(key, value)| {
                match other.0.get(key) {
                    Some(other_value) => {
                        match value.call_method1(py, "__eq__", (other_value,)) {
                            Ok(result) => result.extract::<bool>(py).unwrap_or(false),
                            Err(_) => false,
                        }
                    },
                    None => false,
                }
            })
        })
    }
}

impl Eq for Attributes {}

impl Hash for Attributes {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // 确保相同的字典产生相同的哈希值
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(k1, _), (k2, _)| k1.cmp(k2));

        Python::with_gil(|py| {
            for (key, value) in entries {
                key.hash(state);
                // 使用 Python 对象的 __hash__ 方法
                match value.call_method0(py, "__hash__") {
                    Ok(hash_result) => {
                        if let Ok(hash_value) = hash_result.extract::<isize>(py) {
                            hash_value.hash(state);
                        }
                    },
                    Err(_) => {
                        // 如果对象不可哈希，使用默认值
                        0isize.hash(state);
                    }
                }
            }
        });
    }
}

impl Label for Attributes {}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Node {
    id: usize,
    attributes: Attributes,
}

impl std::fmt::Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node({})", self.id)
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Edge {
    source: usize,
    target: usize,
    attributes: Attributes,
}

#[derive(Clone)]
#[pyclass]
pub struct NetworkXGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_indices: HashMap<String, usize>,
    same_label_fn: Option<SharedRustFn>,
}

fn convert_to_string(obj: &PyObject) -> PyResult<String> {
    Python::with_gil(|py| {
        // Try direct conversion first
        obj.call_method0(py, "__str__")?.extract::<String>(py)
            .or_else(|_| {
                // If that fails, try to convert to a string using repr
                obj.call_method0(py, "__repr__")?.extract::<String>(py)
            })
    })
}

#[pymethods]
impl NetworkXGraph {
    #[new]
    fn new() -> Self {
        NetworkXGraph {
            nodes: Vec::new(),
            edges: Vec::new(),
            node_indices: HashMap::new(),
            same_label_fn: None,
        }
    }

    // 从NetworkX图转换的静态方法
    #[staticmethod]
    fn from_networkx(nx_graph: &Bound<'_, PyAny>) -> PyResult<Self> {
        let nodes = nx_graph.getattr("nodes")?.call_method1("items", ())?;
        let edges = nx_graph.getattr("edges")?.call_method1("data", ())?;

        let mut graph = NetworkXGraph::new();

        for node in nodes.try_iter()? {
            let node = node?;
            let id = node.call_method1("__getitem__", (0, ))?.extract::<PyObject>()?;
            let id = convert_to_string(&id)?;
            let attrs = node.call_method1("__getitem__", (1, ))?.extract::<HashMap<String, PyObject>>()?;
            graph.add_node(id, attrs);
        }
        for edge in edges.try_iter()? {
            let edge = edge?;
            let source = edge.call_method1("__getitem__", (0, ))?.extract::<PyObject>()?;
            let source = convert_to_string(&source)?;
            let target = edge.call_method1("__getitem__", (1, ))?.extract::<PyObject>()?;
            let target = convert_to_string(&target)?;
            let attrs = edge.call_method1("__getitem__", (2, ))?.extract::<HashMap<String, PyObject>>()?;
            graph.add_edge(source, target, attrs);
        }
        
        Ok(graph)
    }

    // 转回NetworkX图的方法
    fn to_networkx(&self, py: Python) -> PyResult<PyObject> {
        let nx = py.import("networkx")?;
        let graph = nx.getattr("Graph")?.call0()?;

        // 添加节点
        for node in &self.nodes {
            let attrs_dict = PyDict::new(py);
            for (k, v) in &node.attributes.0 {
                attrs_dict.set_item(k, v.clone_ref(py))?;
            }
            graph.call_method1(
                "add_node",
                (node.id.clone(), attrs_dict),
            )?;
        }

        // 添加边
        for edge in &self.edges {
            let attrs_dict = PyDict::new(py);
            for (k, v) in &edge.attributes.0 {
                attrs_dict.set_item(k, v.clone_ref(py))?;
            }
            graph.call_method1(
                "add_edge",
                (
                    edge.source.clone(),
                    edge.target.clone(),
                    attrs_dict,
                ),
            )?;
        }

        Ok(graph.into())
    }

    // 其他有用的方法
    fn add_node(&mut self, id: String, attributes: HashMap<String, PyObject>) {
        let index = self.nodes.len();
        self.node_indices.insert(id.clone(), index);
        let attributes = Attributes(attributes);
        self.nodes.push(Node { id: index, attributes });
    }

    fn add_edge(
        &mut self,
        source: String,
        target: String,
        attributes: HashMap<String, PyObject>,
    ) {
        let attributes = Attributes(attributes);
        let source = *self.node_indices.get(&source).unwrap();
        let target = *self.node_indices.get(&target).unwrap();
        self.edges.push(Edge {
            source,
            target,
            attributes,
        });
    }

    fn node_count(&self) -> usize {
        self.nodes.len()
    }

    fn edge_count(&self) -> usize {
        self.edges.len()
    }

    // 获取节点属性
    fn get_node_attributes(&self, node_id: &str) -> Option<HashMap<String, PyObject>> {
        self.node_indices.get(node_id).map(|&index| {
            Python::with_gil(|py| {
                self.nodes[index].attributes.0.iter()
                    .map(|(k, v)| (k.clone(), v.clone_ref(py)))
                    .collect()
            })
        })
    }

    // 获取边属性
    fn get_edge_attributes(
        &self,
        source: &str,
        target: &str,
    ) -> Option<HashMap<String, PyObject>> {
        self.edges
            .iter()
            .find(|e| e.source == *self.node_indices.get(source).unwrap() 
                            && e.target == *self.node_indices.get(target).unwrap())
            .map(|e| Python::with_gil(|py| {
                e.attributes.0.iter()
                    .map(|(k, v)| (k.clone(), v.clone_ref(py)))
                    .collect()
            }))
    }
}

impl<'a> Graph<'a> for NetworkXGraph {
    type Node = Node;

    type Edge = Edge;

    fn nodes(&'a self) -> impl Iterator<Item = &'a Self::Node> {
        self.nodes.iter()
    }

    fn edges(&'a self) -> impl Iterator<Item = &'a Self::Edge> {
        self.edges.iter()
    }

    fn get_edges_pair(&'a self) -> impl Iterator<Item = (&'a Self::Node, &'a Self::Node)> {
        let id_map: HashMap<_, _, std::collections::hash_map::RandomState> = HashMap::from_iter(self.nodes.iter().map(|node| (node.id, node)));
        self.edges.iter().map(|edge| (id_map.get(&edge.source).unwrap().clone(), id_map.get(&edge.target).unwrap().clone()) ).collect::<Vec<_>>().into_iter()
    }

    fn add_node(&mut self, node: Self::Node) {
        let index = self.nodes.len();
        self.node_indices.insert(format!("Node{}.", index), index);
        self.nodes.push(node);
    }

    fn add_edge(&mut self, edge: Self::Edge) {
        self.edges.push(edge);
    }
}

fn test_eq(a: &Py<PyAny>, b: &Py<PyAny>) -> bool {
    Python::with_gil(|py| {
        match a.call_method1(py, "__eq__", (b,)) {
            Ok(result) => result.extract::<bool>(py).unwrap_or(false),
            Err(_) => false
        }
    })
}

fn native_same_label_fn(a: &Attributes, b: &Attributes) -> bool {
    for (k, v) in &a.0 {
        if let Some(other_v) = b.0.get(k) {
            if test_eq(v, other_v) {
                continue;
            }
            // println!("{} != {}", v, other_v);
        } else {
            return false;
        }
    }
    true
}

impl<'a> Labeled<'a> for NetworkXGraph {
    fn label_same(&self, node: &Self::Node, label: &Self::Node) -> bool {
        self.same_label_fn.as_ref().map_or(native_same_label_fn(&node.attributes, &label.attributes), |f| f(&node.attributes, &label.attributes))
    }

    fn get_label(&'a self, node: &'a Self::Node) -> &'a impl Label {
        &node.attributes
    }
}

impl std::fmt::Display for NetworkXGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "NetworkXGraph(\n")?;
        write!(f, "Nodes: [\n")?;
        for node in &self.nodes {
            write!(f, "  {},\n", node)?;
        }
        write!(f, "],\n")?;
        write!(f, "Edges: [\n")?;
        for edge in &self.edges {
            write!(f, "{} -> {}, ", edge.source, edge.target)?;
        }
        write!(f, "]\n")?;
        write!(f, ")")
    }
}

fn to_nx_node(py: Python, node: &Node) -> PyResult<PyObject> {
    let attrs_dict = PyDict::new(py);
    for (k, v) in &node.attributes.0 {
        attrs_dict.set_item(k, v.clone_ref(py))?;
    }
    let nx = py.import("networkx")?;
    let node = nx.getattr("Node")?.call1((node.id.clone(), attrs_dict))?;
    Ok(node.into())
}

#[pyfunction]
pub fn get_simulation_inter(nx_graph1: &Bound<'_, PyAny>, nx_graph2: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    let graph1 = NetworkXGraph::from_networkx(nx_graph1)?;
    let graph2 = NetworkXGraph::from_networkx(nx_graph2)?;


    let sim = graph1.get_simulation_inter(&graph2);

    // Convert simulation to a list of pairs (i, j) where i is a node in graph1, j is a node in graph2
    Python::with_gil(|py| {
        let map = PyDict::new(py);
        
        for (node, set) in sim.iter() {
            let py_set = PySet::new(py, set.iter().map(|node| to_nx_node(py, node)).collect::<PyResult<Vec<_>>>()?)?;
            map.set_item(to_nx_node(py, node)?, py_set)?;
        }
    
        Ok(map.into())
    })
}

#[pyfunction]
pub fn is_simulation_isomorphic(nx_graph1: &Bound<'_, PyAny>, nx_graph2: &Bound<'_, PyAny>) -> PyResult<bool> {
    let graph1 = NetworkXGraph::from_networkx(nx_graph1)?;
    // println!("{}", graph1);
    let graph2 = NetworkXGraph::from_networkx(nx_graph2)?;
    Ok(NetworkXGraph::has_simulation(graph1.get_simulation_inter(&graph2)))
}

impl Directed for NetworkXGraph {}

impl Adjacency<'_> for NetworkXGraph {}

impl AdjacencyInv<'_> for NetworkXGraph {}

// 模块定义
// #[pymodule]
// pub fn networkx_graph(_py: Python, m: &PyModule) -> PyResult<()> {
//     m.add_class::<NetworkXGraph>()?;
//     Ok(())
// }
