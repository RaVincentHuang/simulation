use graph_simulation::graph::base::Graph;
use pyo3::prelude::*;
use graph_simulation::graph::labeled_graph::{Label, Labeled};
use graph_simulation::graph::base::{Directed, Adjacency, AdjacencyInv};


use pyo3::prelude::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

type SharedRustFn = Arc<dyn Fn(&Attributes, &Attributes) -> bool + Send + Sync>;


// 自定义图结构
#[pyclass]
#[derive(Clone)]
pub struct NetworkXGraph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    node_indices: HashMap<String, usize>,
    same_label_fn: Option<SharedRustFn>,
}

#[derive(Clone, Debug)]
struct Attributes(HashMap<String, PyObject>);

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

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct Edge {
    source: usize,
    target: usize,
    attributes: Attributes,
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
    fn from_networkx(py: Python, nx_graph: &PyAny) -> PyResult<Self> {
        let mut graph = NetworkXGraph::new();
        
        // 获取节点及其属性
        let nodes: Vec<(String, PyObject)> = nx_graph
            .call_method0("nodes")?
            .call_method1("items", ())?
            .extract()?;

        for (node_id, attrs) in nodes {
            let attributes: HashMap<String, PyObject> = attrs.extract(py)?;
            graph.add_node(node_id.clone(), attributes);
        }

        // 获取边及其属性
        let edges: Vec<(String, String, PyObject)> = nx_graph
            .call_method0("edges")?
            .call_method1("data", ())?
            .extract()?;

        for (source, target, attrs) in edges {
            let attributes: HashMap<String, PyObject> = attrs.extract(py)?;
            graph.add_edge(source, target, attributes);
        }

        Ok(graph)
    }

    // 转回NetworkX图的方法
    fn to_networkx(&self, py: Python) -> PyResult<PyObject> {
        let nx = py.import("networkx")?;
        let graph = nx.call_method0("Graph")?;

        // 添加节点
        for node in &self.nodes {
            graph.call_method1(
                "add_node",
                (node.id.clone(), node.attributes.0.clone()),
            )?;
        }

        // 添加边
        for edge in &self.edges {
            graph.call_method1(
                "add_edge",
                (
                    edge.source.clone(),
                    edge.target.clone(),
                    edge.attributes.0.clone(),
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
            self.nodes[index].attributes.0.clone()
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
            .map(|e| e.attributes.0.clone())
    }
}

impl<'a> Graph<'a> for NetworkXGraph {
    type Node = Node;

    type Edge = Edge;

    fn nodes(&'a self) -> impl Iterator<Item = &Self::Node> {
        self.nodes.iter()
    }

    fn edges(&'a self) -> impl Iterator<Item = &Self::Edge> {
        self.edges.iter()
    }

    fn get_edges_pair(&'a self) -> impl Iterator<Item = (&Self::Node, &Self::Node)> {
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

impl<'a> Labeled<'a> for NetworkXGraph {
    fn label_same(&self, node: &Self::Node, label: &Self::Node) -> bool {
        self.same_label_fn.as_ref().map_or(node == label, |f| f(&node.attributes, &label.attributes))
    }

    fn get_label(&'a self, node: &'a Self::Node) -> &'a impl Label {
        &node.attributes
    }
}

impl Directed for NetworkXGraph {}

impl Adjacency<'_> for NetworkXGraph {}

impl AdjacencyInv<'_> for NetworkXGraph {}

// 模块定义
#[pymodule]
pub fn networkx_graph(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<NetworkXGraph>()?;
    Ok(())
}
