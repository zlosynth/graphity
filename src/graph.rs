use core::hash::Hash;
use hashbrown::{HashMap, HashSet};

use crate::node::{ConsumerIndex, NodeClass, NodeIndex, ProducerIndex};

pub struct Graph<N, NI, CI, PI>
where
    NI: Hash,
    CI: Hash,
    PI: Hash,
{
    index_counter: usize,
    pub nodes: HashMap<NI, N>,
    pub edges: HashSet<(PI, CI)>,
}

#[derive(Debug)]
pub enum AddEdgeError {
    OccupiedConsumer,
}

#[allow(clippy::new_without_default)]
impl<N, NI, CI, PI> Graph<N, NI, CI, PI>
where
    N: NodeClass,
    NI: NodeIndex<Class = N::Class>,
    CI: ConsumerIndex<NodeIndex = NI, Consumer = NI::Consumer>,
    PI: ProducerIndex<NodeIndex = NI, Producer = NI::Producer>,
{
    pub fn new() -> Self {
        Self {
            index_counter: 0,
            nodes: HashMap::new(),
            edges: HashSet::new(),
        }
    }

    pub fn add_node<IntoN>(&mut self, node: IntoN) -> NI
    where
        IntoN: Into<N>,
    {
        let node = node.into();
        let index = NI::new(node.class(), self.index_counter);
        self.nodes.insert(index, node);
        self.index_counter += 1;
        index
    }

    pub fn remove_node(&mut self, node_index: NI) {
        self.nodes.remove(&node_index);
        self.edges.retain(|(producer, consumer)| {
            producer.node_index() != node_index && consumer.node_index() != node_index
        });
    }

    pub fn node(&self, node_index: &NI) -> Option<&N> {
        self.nodes.get(node_index)
    }

    pub fn node_mut(&mut self, node_index: &NI) -> Option<&mut N> {
        self.nodes.get_mut(node_index)
    }

    pub fn add_edge(&mut self, producer: PI, consumer: CI) -> Result<(), AddEdgeError> {
        self.edges
            .iter()
            .try_for_each(|(existing_producer, existing_consumer)| {
                if *existing_consumer == consumer && *existing_producer != producer {
                    return Err(AddEdgeError::OccupiedConsumer);
                }
                Ok(())
            })?;
        self.edges.insert((producer, consumer));
        Ok(())
    }

    pub fn must_add_edge(&mut self, producer: PI, consumer: CI) {
        self.add_edge(producer, consumer).unwrap();
    }

    pub fn remove_edge(&mut self, producer: PI, consumer: CI) {
        self.edges.remove(&(producer, consumer));
    }

    pub fn has_edge(&mut self, producer: PI, consumer: CI) -> bool {
        self.edges.contains(&(producer, consumer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{CommonConsumerIndex, CommonProducerIndex};

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestNode(i32);

    impl NodeClass for TestNode {
        type Class = TestClass;

        fn class(&self) -> Self::Class {
            TestClass
        }
    }

    impl From<i32> for TestNode {
        fn from(n: i32) -> Self {
            Self(n)
        }
    }

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestNodeIndex {
        index: usize,
    }

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestClass;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestConsumer;

    type TestConsumerIndex = CommonConsumerIndex<TestNodeIndex>;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestProducer;

    type TestProducerIndex = CommonProducerIndex<TestNodeIndex>;

    impl NodeIndex for TestNodeIndex {
        type Class = TestClass;
        type Consumer = TestConsumer;
        type ConsumerIndex = TestConsumerIndex;
        type Producer = TestProducer;
        type ProducerIndex = TestProducerIndex;

        fn new(_class: Self::Class, index: usize) -> Self {
            Self { index }
        }

        fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
        where
            IntoC: Into<Self::Consumer>,
        {
            CommonConsumerIndex::new(*self, consumer.into())
        }

        fn producer<IntoP>(&self, producer: IntoP) -> TestProducerIndex
        where
            IntoP: Into<TestProducer>,
        {
            CommonProducerIndex::new(*self, producer.into())
        }
    }

    type TestGraph = Graph<TestNode, TestNodeIndex, TestConsumerIndex, TestProducerIndex>;

    #[test]
    fn initialize_node_index() {
        let _node_index = TestNodeIndex::new(TestClass, 0);
    }

    #[test]
    fn get_consumer_index() {
        let node_index = TestNodeIndex::new(TestClass, 0);

        let _consumer_index = node_index.consumer(TestConsumer);
    }

    #[test]
    fn get_consumer_index_node_index() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let consumer_index = node_index.consumer(TestConsumer);

        assert_eq!(consumer_index.node_index(), node_index)
    }

    #[test]
    fn get_consumer_index_consumer() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let consumer_index = node_index.consumer(TestConsumer);

        assert_eq!(consumer_index.consumer(), TestConsumer)
    }

    #[test]
    fn get_producer_index() {
        let node_index = TestNodeIndex::new(TestClass, 0);

        let _producer_index = node_index.producer(TestProducer);
    }

    #[test]
    fn get_producer_index_node_index() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let producer_index = node_index.producer(TestProducer);

        assert_eq!(producer_index.node_index(), node_index)
    }

    #[test]
    fn get_producer_index_producer() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let producer_index = node_index.producer(TestProducer);

        assert_eq!(producer_index.producer(), TestProducer)
    }

    #[test]
    fn initialize_graph() {
        let _graph = TestGraph::new();
    }

    #[test]
    fn add_node() {
        let mut graph = TestGraph::new();

        graph.add_node(10);
    }

    #[test]
    fn remove_node() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        graph.remove_node(index);

        assert!(graph.node(&index).is_none());
    }

    #[test]
    fn get_node() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        assert_eq!(*graph.node(&index).unwrap(), TestNode(10));
    }

    #[test]
    fn return_none_on_get_nonexistent_node() {
        let graph = TestGraph::new();

        assert!(graph.node(&NodeIndex::new(TestClass, 100)).is_none());
    }

    #[test]
    fn get_node_mut() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        *graph.node_mut(&index).unwrap() = 20.into();
        assert_eq!(*graph.node(&index).unwrap(), 20.into());
    }

    #[test]
    fn return_none_on_get_nonexistent_node_mut() {
        let mut graph = TestGraph::new();

        assert!(graph.node_mut(&NodeIndex::new(TestClass, 100)).is_none());
    }

    #[test]
    fn add_edge() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));

        graph.must_add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        assert!(graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn add_multiple_edges_with_single_source() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);

        graph.must_add_edge(one.producer(TestProducer), two.consumer(TestConsumer));
        graph.must_add_edge(one.producer(TestProducer), three.consumer(TestConsumer));

        assert!(graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
        assert!(graph.has_edge(one.producer(TestProducer), three.consumer(TestConsumer)));
    }

    #[test]
    fn return_error_on_add_multiple_edges_with_single_destination() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);

        graph.must_add_edge(one.producer(TestProducer), three.consumer(TestConsumer));
        assert!(graph
            .add_edge(two.producer(TestProducer), three.consumer(TestConsumer))
            .is_err());
    }

    #[test]
    fn remove_edge() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        graph.must_add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        graph.remove_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn remove_edges_on_source_node_removal() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        graph.must_add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        graph.remove_node(one);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn remove_edges_on_destination_node_removal() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);
        graph.must_add_edge(one.producer(TestProducer), two.consumer(TestConsumer));
        graph.must_add_edge(one.producer(TestProducer), three.consumer(TestConsumer));

        graph.remove_node(two);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
        assert!(graph.has_edge(one.producer(TestProducer), three.consumer(TestConsumer)));
    }
}
