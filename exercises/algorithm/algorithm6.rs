/*
	dfs
	This problem requires you to implement a basic DFS traversal
*/

use std::collections::HashSet;

struct Graph {
    adj: Vec<Vec<usize>>, 
}

impl Graph {
    fn new(n: usize) -> Self {
        Graph {
            adj: vec![vec![]; n],
        }
    }

    fn add_edge(&mut self, src: usize, dest: usize) {
        self.adj[src].push(dest);
        self.adj[dest].push(src); 
    }

    fn dfs_util(&self, v: usize, visited: &mut HashSet<usize>, visit_order: &mut Vec<usize>) {
        // insert returns false if the vertex was already visited.
        if !visited.insert(v) {
            return;
        }
        visit_order.push(v);

        for &neighbor in &self.adj[v] {
            self.dfs_util(neighbor, visited, visit_order);
        }
    }

    // Perform a depth-first search on the graph, return the order of visited nodes
    fn dfs(&self, start: usize) -> Vec<usize> {
        let mut visited = HashSet::new();
        let mut visit_order = Vec::new(); 
        // An invalid start has no traversal, including in an empty graph.
        if start >= self.adj.len() {
            return visit_order;
        }
        self.dfs_util(start, &mut visited, &mut visit_order);
        visit_order
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dfs_branch_order() {
        let mut graph = Graph::new(5);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(1, 3);
        graph.add_edge(2, 4);

        assert_eq!(graph.dfs(0), vec![0, 1, 3, 2, 4]);
    }

    #[test]
    fn test_dfs_duplicate_edges() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);

        assert_eq!(graph.dfs(0), vec![0, 1, 2]);
    }

    #[test]
    fn test_dfs_single_node() {
        assert_eq!(Graph::new(1).dfs(0), vec![0]);
    }

    #[test]
    fn test_dfs_invalid_start() {
        assert!(Graph::new(0).dfs(0).is_empty());
        assert!(Graph::new(3).dfs(3).is_empty());
    }

    #[test]
    fn test_dfs_simple() {
        let mut graph = Graph::new(3);
        graph.add_edge(0, 1);
        graph.add_edge(1, 2);

        let visit_order = graph.dfs(0);
        assert_eq!(visit_order, vec![0, 1, 2]);
    }

    #[test]
    fn test_dfs_with_cycle() {
        let mut graph = Graph::new(4);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(1, 2);
        graph.add_edge(2, 3);
        graph.add_edge(3, 3); 

        let visit_order = graph.dfs(0);
        assert_eq!(visit_order, vec![0, 1, 2, 3]);
    }

    #[test]
    fn test_dfs_disconnected_graph() {
        let mut graph = Graph::new(5);
        graph.add_edge(0, 1);
        graph.add_edge(0, 2);
        graph.add_edge(3, 4); 

        let visit_order = graph.dfs(0);
        assert_eq!(visit_order, vec![0, 1, 2]); 
        let visit_order_disconnected = graph.dfs(3);
        assert_eq!(visit_order_disconnected, vec![3, 4]); 
    }
}
