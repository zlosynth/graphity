#![feature(alloc)]

extern crate alloc;

use alloc::vec::Vec;

pub struct NodeIndex(usize);

pub struct Graph {
    nodes: Vec<Option<u32>>,
}

impl Graph {
    pub fn new() -> Self {
        Self { nodes: Vec::new() }
    }

    pub fn add_node(&mut self, weight: u32) -> NodeIndex {
        let empty_index = self
            .nodes
            .iter()
            .enumerate()
            .find(|&(_, x)| x.is_none())
            .map(|x| x.0);

        let node_index = if let Some(empty_index) = empty_index {
            self.nodes[empty_index] = Some(weight);
            empty_index
        } else {
            self.nodes.push(Some(weight));
            self.nodes.len() - 1
        };

        NodeIndex(node_index)
    }

    pub fn node_weight(&self, index: &NodeIndex) -> &u32 {
        self.nodes[index.0].as_ref().unwrap()
    }

    pub fn node_weight_mut(&mut self, index: &NodeIndex) -> &mut u32 {
        self.nodes[index.0].as_mut().unwrap()
    }

    pub fn remove_node(&mut self, index: NodeIndex) -> u32 {
        self.nodes[index.0].take().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_graph() {
        let _graph = Graph::new();
    }

    #[test]
    fn add_node() {
        let mut graph = Graph::new();

        let _node_index = graph.add_node(1);
    }

    #[test]
    fn read_node() {
        let mut graph = Graph::new();
        let node_index = graph.add_node(1);

        assert_eq!(graph.node_weight(&node_index), &1);
    }

    #[test]
    fn edit_node() {
        let mut graph = Graph::new();
        let node_index = graph.add_node(1);

        *graph.node_weight_mut(&node_index) = 2;
        assert_eq!(graph.node_weight(&node_index), &2);
    }

    #[test]
    fn remove_node() {
        let mut graph = Graph::new();
        let node_index = graph.add_node(1);

        assert_eq!(graph.remove_node(node_index), 1);
    }

    #[test]
    fn repeatedly_add_and_remove_nodes() {
        let mut graph = Graph::new();

        let node_index_1 = graph.add_node(1);
        let node_index_2 = graph.add_node(2);
        assert_eq!(graph.remove_node(node_index_1), 1);
        assert_eq!(graph.remove_node(node_index_2), 2);
        let node_index_3 = graph.add_node(3);
        assert_eq!(graph.remove_node(node_index_3), 3);
    }
}
