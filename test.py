import simulation

from simulation import *
query = Hypergraph()
query.add_node("a") # id 0
query.add_node("b") # id 1
query.add_node("c") # id 2
query.add_hyperedge(Hyperedge({0, 1}, "e1", 0))
query.add_hyperedge(Hyperedge({1, 2}, "e2", 1))
query.add_hyperedge(Hyperedge({0, 2}, "e3", 2))

data = Hypergraph()
data.add_node("a")
data.add_node("b")
data.add_node("c")
data.add_hyperedge(Hyperedge({0, 1}, "e1", 0))
data.add_hyperedge(Hyperedge({1, 2}, "e2", 1))
data.add_hyperedge(Hyperedge({0, 2}, "e3", 2))

# register type_same

query_vertices = []
data_vertices = []

m1= {}

for u in query_vertices:
    for v in data_vertices:
        # nli,
        m1[u, v] = True

query.set_type_same_fn(lambda x_id, y_id: m1[x_id, y_id])

delta = Delta()

likely_vertices = []
m2 = {}

for (u, v) in likely_vertices:
    pairs = []
    for pair in pairs:
        delta.add_sematic_cluster_pair(u, v, pair[0], pair[1])
        



matches = {}
for (u, v), pair in m2.items():
    matches[u, v] = [(1, 2)]
    
d_match = DMatch.from_dict(matches)



sim = Hypergraph.get_hyper_simulation(query, data, delta, d_match)

# set[tuple[int, int]]