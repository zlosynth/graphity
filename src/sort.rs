use alloc::collections::VecDeque;
use alloc::vec::Vec;
use core::hash::Hash;
use hashbrown::HashSet;

#[derive(Debug, Clone, Copy)]
pub struct Cycle;

pub fn topological_sort<N, NI, EI>(nodes: NI, edges: EI) -> Result<Vec<N>, Cycle>
where
    N: Copy + Hash + Eq,
    NI: IntoIterator<Item = N>,
    EI: IntoIterator<Item = (N, N)>,
{
    let mut sorted_nodes: Vec<N> = Vec::new();

    let mut edges: HashSet<(N, N)> = edges.into_iter().collect();

    let mut nodes_queue: VecDeque<N> = {
        let nodes: HashSet<_> = nodes.into_iter().collect();
        let consumers: HashSet<_> = edges.iter().map(|(_, consumer)| *consumer).collect();
        let pure_producers = nodes.difference(&consumers);
        pure_producers.copied().collect()
    };

    while let Some(node) = nodes_queue.pop_front() {
        sorted_nodes.push(node);

        let outcoming_edges: HashSet<_> = edges
            .iter()
            .filter(|(source, _)| *source == node)
            .copied()
            .collect();

        outcoming_edges.into_iter().for_each(|edge| {
            let remote_node = edge.1;

            edges.remove(&edge);

            let no_other_incoming_edges = edges
                .iter()
                .filter(|(_, destination)| *destination == remote_node)
                .count()
                == 0;
            if no_other_incoming_edges {
                nodes_queue.push_back(remote_node);
            }
        });
    }

    if edges.is_empty() {
        Ok(sorted_nodes)
    } else {
        Err(Cycle)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use alloc::vec;

    ///     [0]  [1]
    ///     / \       |
    ///  [2]   [3]    |
    ///     \ /  \    V
    ///     [4]  [5]
    #[test]
    fn sort() {
        let nodes: HashSet<_> = (0..=5).collect();
        let edges = vec![(3, 5), (2, 4), (0, 2), (3, 4), (0, 3)];
        let mut first_group: HashSet<_> = vec![0, 1].into_iter().collect();
        let mut second_group: HashSet<_> = vec![2, 3].into_iter().collect();
        let mut third_group: HashSet<_> = vec![4, 5].into_iter().collect();

        let sorted_nodes = match topological_sort(nodes, edges) {
            Ok(sorted_nodes) => sorted_nodes,
            Err(_) => panic!("Failed sorting nodes"),
        };

        assert_eq!(sorted_nodes.len(), 6);

        let mut iter = sorted_nodes.into_iter();

        let item = iter.next().unwrap();
        assert!(first_group.contains(&item));
        first_group.remove(&item);

        let item = iter.next().unwrap();
        assert!(first_group.contains(&item));
        first_group.remove(&item);

        let item = iter.next().unwrap();
        assert!(second_group.contains(&item));
        second_group.remove(&item);

        let item = iter.next().unwrap();
        assert!(second_group.contains(&item));
        second_group.remove(&item);

        let item = iter.next().unwrap();
        assert!(third_group.contains(&item));
        third_group.remove(&item);

        let item = iter.next().unwrap();
        assert!(third_group.contains(&item));
        third_group.remove(&item);
    }

    ///      [0]
    ///  |  /   \  A
    ///  | |     | |
    ///  V  \   /  |
    ///      [1]
    #[test]
    fn find_cycle() {
        let nodes: HashSet<_> = vec![0, 1].into_iter().collect();
        let edges = vec![(0, 1), (1, 0)];

        match topological_sort(nodes, edges) {
            Err(Cycle) => (),
            Ok(_) => panic!("Must fail"),
        }
    }
}
