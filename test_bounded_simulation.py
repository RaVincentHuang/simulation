#!/usr/bin/env python3
"""
Test script for bounded simulation functionality.
"""

import networkx as nx
import sys
sys.path.insert(0, '/home/vincent/simulation/target/release')

try:
    import simulation
    print("✓ Successfully imported simulation module")
except ImportError as e:
    print(f"✗ Failed to import simulation module: {e}")
    sys.exit(1)

def test_bounded_simulation():
    """Test the get_bounded_simulation function."""
    
    # Create two simple graphs
    graph1 = nx.DiGraph()
    graph1.add_node(0, label='A')
    graph1.add_node(1, label='B')
    graph1.add_edge(0, 1)
    
    graph2 = nx.DiGraph()
    graph2.add_node(0, label='A')
    graph2.add_node(1, label='B')
    graph2.add_node(2, label='B')
    graph2.add_edge(0, 1)
    graph2.add_edge(0, 2)
    
    # Define the compare function (checks if labels are the same)
    def compare_labels(attr1, attr2):
        return attr1.get('label') == attr2.get('label')
    
    # Define the bound function (returns a fixed bound value for each node)
    def get_bound_value(attr):
        return 2  # bound value = 2 for all nodes
    
    try:
        result = simulation.get_bounded_simulation(
            graph1, 
            graph2, 
            compare_labels, 
            get_bound_value,
            is_label_cached=False
        )
        
        print("✓ get_bounded_simulation executed successfully")
        print(f"  Result type: {type(result)}")
        
        # Check if result is a dictionary
        if isinstance(result, dict):
            print("✓ Result is a dictionary")
            print(f"  Result size: {len(result)} entries")
            for i, (node, sim_set) in enumerate(result.items()):
                if i < 5:  # Print only first 5 entries
                    print(f"  Node: {node}, Simulated nodes: {list(sim_set) if hasattr(sim_set, '__iter__') else sim_set}")
        else:
            print(f"✗ Expected dict, got {type(result)}")
            return False
        
        return True
        
    except Exception as e:
        print(f"✗ Error calling get_bounded_simulation: {e}")
        import traceback
        traceback.print_exc()
        return False


if __name__ == "__main__":
    print("Testing bounded simulation functionality...")
    print("=" * 50)
    
    success = test_bounded_simulation()
    
    print("=" * 50)
    if success:
        print("✓ All tests passed!")
    else:
        print("✗ Some tests failed")
        sys.exit(1)
