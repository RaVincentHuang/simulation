import simulation
from simulation import Hypergraph, Node, Hyperedge

def hyperedge_same_fn(hyperedge1: Hyperedge, hyperedge2: Hyperedge) -> bool:
    """
    Check if two hyperedges are the same based on their IDs.
    """
    return len(hyperedge1.id_set()) == len(hyperedge2.id_set())

# Callable[[Hyperedge, Hyperedge], dict[int, set[int]]]
def l_match_fn(hyperedge1: Hyperedge, hyperedge2: Hyperedge) -> dict[int, set[int]]:
    """
    Match hyperedges based on their IDs.
    Returns a mapping of node IDs from hyperedge1 to hyperedge2.
    """
    mapping = {}
    for id1 in hyperedge1.id_set():
        for id2 in hyperedge2.id_set():
            if id1 == id2:
                mapping[id1] = {id2}
    return mapping

def label_same_fn(desc1: str, desc2: str) -> bool:
    """
    Check if two nodes are the same based on their descriptions.
    """
    return desc1 == desc2 

if __name__ == "__main__":
    # Create a hypergraph
    query = Hypergraph()
    query.add_node("a") # id 0
    query.add_node("b") # id 1
    query.add_node("c") # id 2
    query.add_hyperedge(Hyperedge({0, 1}, "e1", 0))
    query.add_hyperedge(Hyperedge({1, 2}, "e2", 1))
    query.add_hyperedge(Hyperedge({0, 2}, "e3", 2))
    query.set_type_same_fn(label_same_fn) # desc
    query.set_l_predicate_fn(hyperedge_same_fn)
    
    data = Hypergraph()
    data.add_node("a")
    data.add_node("b")
    data.add_node("c")
    data.add_hyperedge(Hyperedge({0, 1}, "e1", 0))
    data.add_hyperedge(Hyperedge({1, 2}, "e2", 1))
    data.add_hyperedge(Hyperedge({0, 2}, "e3", 2))
    
    print("Start hyper simulation")
    
    print(Hypergraph.hyper_simulation(query, data, l_match_fn))
