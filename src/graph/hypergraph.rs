use std::{collections::{HashMap, HashSet}, fmt::Display, hash::Hash};

use pyo3::prelude::*;
use graph_simulation::algorithm::hyper_simulation::{HyperSimulation, LMatch, LPredicate};
use graph_base::interfaces::{edge, graph::{self, SingleId}, hypergraph::{self, ContainedHyperedge}, typed, vertex};
// use graph_base::interfaces::hypergraph;

#[derive(Clone, Debug, Eq)]
#[pyclass(name = "Node")]
pub struct Node {
    id: usize,
    desc: String,
}

impl Hash for Node {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[pymethods]
impl Node {
    #[new]
    pub fn new(id: usize, desc: String) -> Self {
        Node { id, desc }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn desc(&self) -> &String {
        &self.desc
    }
}

#[derive(Clone, Debug, Eq)]
#[pyclass(name = "Hyperedge")]
pub struct Hyperedge {
    id_set: HashSet<usize>,
    desc: String,
    id: usize, 
}

impl PartialEq for Hyperedge {
    fn eq(&self, other: &Self) -> bool {
        self.id_set == other.id_set
    }
}

impl Hash for Hyperedge {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for id in &self.id_set {
            id.hash(state);
        }    
    }
}

#[pymethods]
impl Hyperedge {
    #[new]
    pub fn new(id_set: HashSet<usize>, desc: String, id: usize) -> Self {
        Hyperedge { id_set, desc, id }
    }

    pub fn id_set(&self) -> Vec<usize> {
        self.id_set.iter().cloned().collect()
    }

    pub fn desc(&self) -> &String {
        &self.desc
    }
}

#[pyclass(name = "Hypergraph")]
pub struct Hypergraph {
    nodes: Vec<Node>,
    hyperedges: Vec<Hyperedge>,
    type_same_fn: Option<Py<PyAny>>, // (str, str) -> bool
    l_predicate_fn: Option<Py<PyAny>>, // (Hyperedge, Hyperedge) -> bool
}

#[pymethods]
impl Hypergraph {
    #[new]
    pub fn new() -> Self {
        Hypergraph {
            nodes: Vec::new(),
            hyperedges: Vec::new(),
            type_same_fn: None,
            l_predicate_fn: None,
        }
    }

    pub fn add_node(&mut self, desc: String) {
        self.nodes.push(Node {
            id: self.nodes.len(),
            desc: desc,
        });
    }

    pub fn add_hyperedge(&mut self, hyperedge: PyRef<Hyperedge>) {
        self.hyperedges.push(Hyperedge {
            id_set: hyperedge.id_set.clone(),
            desc: hyperedge.desc.clone(),
            id: self.hyperedges.len(),
        });
    }

    pub fn set_type_same_fn(&mut self, type_same_fn: Py<PyAny>) {
        self.type_same_fn = Some(type_same_fn);
    }

    pub fn set_l_predicate_fn(&mut self, l_predicate_fn: Py<PyAny>) {
        self.l_predicate_fn = Some(l_predicate_fn);
    }

    pub fn get_node_desc_by_id(&self, id: usize) -> Option<String> {
        self.nodes.get(id).map(|node| node.desc.clone())
    }

    #[staticmethod]
    pub fn hyper_simulation(query: PyRef<Hypergraph>, data: PyRef<Hypergraph>, l_match_fn: Py<PyAny>) -> HashMap<usize, HashSet<usize>> {
        let mut l_match = LMatchImpl::from(l_match_fn);
        let sim = HyperSimulation::get_simulation_naive(&*query, &*data, &mut l_match);
        // Convert HashMap<&Node, HashSet<&Node>> to HashMap<usize, HashSet<usize>>
        sim.into_iter()
            .map(|(k, v)| (k.id(), v.into_iter().map(|n| n.id()).collect()))
            .collect()
    }

    #[staticmethod]
    pub fn soft_hyper_simulation(query: PyRef<Hypergraph>, data: PyRef<Hypergraph>, l_match_fn: Py<PyAny>) -> HashMap<usize, HashSet<usize>> {
        let mut l_match = LMatchImpl::from(l_match_fn);
        let sim = HyperSimulation::get_soft_simulation_naive(&*query, &*data, &mut l_match);
        // Convert HashMap<&Node, HashSet<&Node>> to HashMap<usize, HashSet<usize>>
        sim.into_iter()
            .map(|(k, v)| (k.id(), v.into_iter().map(|n| n.id()).collect()))
            .collect()
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Node(id: {}, desc: {})", self.id, self.desc)
    }
}

impl graph::SingleId for Node {
    fn id(&self) -> usize {
        self.id
    }
}

impl vertex::Vertex for Node {}

impl edge::Hyperedge for Hyperedge {
    fn id_set(&self) -> HashSet<usize> {
        self.id_set.clone()
    }
}

impl hypergraph::IdVector for Hyperedge {
    fn id(&self) -> Vec<usize> {
        self.id_set.iter().cloned().collect()
    }
}

impl<'a> hypergraph::Hypergraph<'a> for Hypergraph {
    type Node = Node;
    type Edge = Hyperedge;

    fn new() -> Self {
        Hypergraph {
            nodes: Vec::new(),
            hyperedges: Vec::new(),
            type_same_fn: None,
            l_predicate_fn: None,
        }
    }

    fn nodes(&'a self) -> impl Iterator<Item = &'a Self::Node> {
        self.nodes.iter()
    }

    fn hyperedges(&'a self) -> impl Iterator<Item = &'a Self::Edge> {
        self.hyperedges.iter()
    }

    fn add_node(&mut self, node: Self::Node) {
        self.nodes.push(node);
    }

    fn add_hyperedge(&mut self, edge: Self::Edge) {
        self.hyperedges.push(edge);
    }

    fn get_node_by_id(&'a self, id: usize) -> Option<&'a Self::Node> {
        if id < self.nodes.len() {
            Some(&self.nodes[id])
        } else {
            None
        }
    }
}

impl<'a> typed::Typed<'a> for Hypergraph {
    fn type_same(&self, x: &Node, y: &Node) -> bool {
        if let Some(type_same_fn) = self.type_same_fn.as_ref() {
            // You cannot pass Rust trait objects directly to Python.
            // Instead, you need to define what data you want to compare and pass only that.
            // For example, if your Type trait has an id() method, you could do:
            Python::with_gil(|py| {
                type_same_fn.call1(py, (&x.desc, &y.desc)).map_or(false, |result| {
                    result.extract::<bool>(py).unwrap_or(false)
                })
            })
        } else {
            false
        }
    }
}

impl<'a> ContainedHyperedge<'a> for Hypergraph {}

#[pyclass(name = "LMatch")]
pub struct LMatchImpl {
    l_match_fn: Option<Py<PyAny>>, // (Hyperedge, Hyperedge) -> dict[int, set[int]]
    l_match_cache: HashMap<(usize, usize), HashMap<usize, HashSet<usize>>>,
    empty_match: HashMap<usize, HashSet<usize>>,
    empty_set: HashSet<usize>,
}

#[pymethods]
impl LMatchImpl {
    #[new]
    pub fn new() -> Self {
        LMatchImpl {
            l_match_fn: None,
            l_match_cache: HashMap::new(),
            empty_match: HashMap::new(),
            empty_set: HashSet::new(),
        }
    }

    #[staticmethod]
    pub fn from(l_match_fn: Py<PyAny>) -> Self {
        LMatchImpl {
            l_match_fn: Some(l_match_fn),
            l_match_cache: HashMap::new(),
            empty_match: HashMap::new(),
            empty_set: HashSet::new(),
        }
    }
}

impl LMatch for LMatchImpl {
    type Edge = Hyperedge;

    fn new() -> Self {
        LMatchImpl {
            l_match_fn: None,
            l_match_cache: HashMap::new(),
            empty_match: HashMap::new(),
            empty_set: HashSet::new(),
        }
    }

    fn l_match_with_node_mut(&mut self, e: &Self::Edge, e_prime: &Self::Edge, u: usize) -> &HashSet<usize> {
        let l_match = self.l_match_cache.entry((e.id, e_prime.id)).or_insert_with(|| {
            if let Some(l_match_fn) = self.l_match_fn.as_ref() {
            Python::with_gil(|py| {
                l_match_fn.call1(py, (e.clone(), e_prime.clone())).map_or_else(
                    |_| HashMap::new(),
                    |result| result.extract::<HashMap<usize, HashSet<usize>>>(py).unwrap_or_default(),
                )
            })
            } else {
                HashMap::new()
            }
        });

        l_match.entry(u).or_insert_with(HashSet::new)
    }

    fn l_match_with_node(&self, e: &Self::Edge, e_prime: &Self::Edge, u: usize) -> &HashSet<usize> {
        if let Some(let_match) = self.l_match_cache.get(&(e.id, e_prime.id)) {
            if let Some(match_set) = let_match.get(&u) {
                return match_set;
            } 
        }
        return &self.empty_set
    }

    fn dom(&self, e: &Self::Edge, e_prime: &Self::Edge) -> impl Iterator<Item = &usize> {
        if let Some(let_match) = self.l_match_cache.get(&(e.id, e_prime.id)) {
            return let_match.keys();
        } else {
            // If the cache does not contain the entry, return an empty iterator
            return self.empty_match.keys();
        }
    }
}

impl<'a> LPredicate<'a> for Hypergraph {
    fn l_predicate_node(&'a self, u: &'a Self::Node, v: &'a Self::Node) -> bool {
        true
    }

    fn l_predicate_edge(&'a self, e: &'a Self::Edge, e_prime: &'a Self::Edge) -> bool {
        if let Some(l_predicate_fn) = self.l_predicate_fn.as_ref() {
            Python::with_gil(|py| {
                l_predicate_fn.call1(py, (e.clone(), e_prime.clone())).map_or(false, |result| {
                    result.extract::<bool>(py).unwrap_or(false)
                })
            })
        } else {
            false
        }
    }

    fn l_predicate_set(&'a self, x: &HashSet<&'a Self::Node>, y: &HashSet<&'a Self::Node>) -> bool {
        // Implement your logic here
        true
    }
}

// fn test_sim(g: &Hypergraph, h: &Hypergraph)  {
//     let mut l_match = LMatchImpl::new();
//     let sim = HyperSimulation::get_simulation_naive(g, h, &mut l_match);
// }