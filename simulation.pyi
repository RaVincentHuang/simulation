
from typing import Callable, Dict, Optional
import networkx


def get_simulation_inter(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, is_label_cached=False) -> Dict: 
    """
    Get the simulation between two graphs.
    """

def is_simulation_isomorphic(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, is_label_cached=False) -> bool:
    """
    Check if two graphs are isomorphic by graph simulation.
    """

def get_simulation_inter_fn(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, compare_fn: Callable, is_label_cached=False) -> Dict: 
    """
    Get the simulation between two graphs.
    """
    
def is_simulation_isomorphic_fn(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, compare_fn: Callable, is_label_cached=False) -> bool:
    """
    Check if two graphs are isomorphic by graph simulation.
    """

def is_simulation_isomorphic_of_node_edge_fn(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, node_compare_fn: Callable,  edge_compare_fn: Callable, is_label_cached=False) -> bool:
    """
    Check if two graphs are isomorphic by graph simulation.
    """

def is_simulation_isomorphic_of_edge_fn(nx_graph1: networkx.DiGraph, nx_graph2: networkx.DiGraph, node_edge_compare_fn: Callable, is_label_cached=False) -> bool:
    """
    Check if two graphs are isomorphic by graph simulation.
    """

class Node:
    """
    Node for hypergraph.
    """
    def __init__(self, id: int, desc: str): ...
    def id(self) -> int: ...
    def desc(self) -> str: ...

class Hyperedge:
    """
    Hyperedge for hypergraph.
    """
    def __init__(self, id_set: set[int], desc: str, id: int): ...
    def id_set(self) -> set[int]: ...
    def desc(self) -> str: ...

class Hypergraph:
    """
    Hypergraph class.
    """
    def __init__(self): ...
    
    def add_node(self, desc: str): ...
    
    def add_hyperedge(self, hyperedge: Hyperedge): ...
    
    def set_type_same_fn(self, type_same_fn: Callable[[str, str], bool]): ... # L(v) = L(u)
    
    def set_l_predicate_fn(self, l_predicate_fn: Callable[[Hyperedge, Hyperedge], bool]): ... # L_P(e1, e2)
    
    def get_node_desc_by_id(self, node_id: int) -> Optional[str]: ...
    
    @staticmethod
    def hyper_simulation(query: 'Hypergraph', data: 'Hypergraph', l_match_fn: Callable[[Hyperedge, Hyperedge], dict[int, set[int]]]) -> dict[int, set[int]]:
        """
        Hyper simulation.
        """
    
    @staticmethod
    def soft_hyper_simulation(query: 'Hypergraph', data: 'Hypergraph', l_match_fn: Callable[[Hyperedge, Hyperedge], dict[int, set[int]]]) -> dict[int, set[int]]:
        """
        Hyper simulation.
        """
