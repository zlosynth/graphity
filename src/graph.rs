use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::node::NodeClass;

pub trait NodeIndex: Copy + Hash + Eq {
    type Class: Copy + Hash + Eq;
    type Consumer: Copy + Hash + Eq;
    // TODO: What about node index association?
    type ConsumerIndex: ConsumerIndexT<NodeIndex = Self>;
    type Producer: Copy + Hash + Eq;

    fn new(class: Self::Class, index: usize) -> Self;
    fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
    where
        IntoC: Into<Self::Consumer>;
    fn producer<IntoP>(&self, producer: IntoP) -> ProducerIndex<Self>
    where
        IntoP: Into<Self::Producer>;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConsumerIndex<NI>
where
    NI: NodeIndex,
{
    node_index: NI,
    consumer: NI::Consumer,
}

pub trait ConsumerIndexT: Copy + Hash + Eq {
    type NodeIndex: NodeIndex;

    fn new(
        node_index: Self::NodeIndex,
        consumer: <<Self as ConsumerIndexT>::NodeIndex as NodeIndex>::Consumer,
    ) -> Self;
    fn node_index(&self) -> Self::NodeIndex;
    fn consumer(&self) -> <<Self as ConsumerIndexT>::NodeIndex as NodeIndex>::Consumer;
}

// TODO: Turn this into a Trait
// 1. DONE implemented here on the struct directly
// 2. Turn other traits to require the trait, not the struct
// 3. Use the trait for Signal
// 4. Consider dropping the default implementation and have it explicit per each
impl<NI> ConsumerIndexT for ConsumerIndex<NI>
where
    NI: NodeIndex,
{
    type NodeIndex = NI;

    fn new(node_index: Self::NodeIndex, consumer: NI::Consumer) -> Self {
        Self {
            node_index,
            consumer,
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        self.node_index
    }

    fn consumer(&self) -> NI::Consumer {
        self.consumer
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ProducerIndex<NI>
where
    NI: NodeIndex,
{
    pub node_index: NI,
    pub producer: NI::Producer,
}

impl<NI> ProducerIndex<NI>
where
    NI: NodeIndex,
{
    pub fn new(node_index: NI, producer: NI::Producer) -> Self {
        Self {
            node_index,
            producer,
        }
    }
}

// Directed graph Each node is further divided into producers and consumers with
// MAX 1 indegree for each consumer and arbitrary number of outdegrees for
// producer all consumers of a node are connected to all producers of the node
// TODO: Use converusmer index as trait
pub struct Graph<N, NI, CI>
where
    N: NodeClass<Class = NI::Class>,
    NI: NodeIndex,
    CI: ConsumerIndexT,
{
    index_counter: usize,
    pub nodes: HashMap<NI, N>,
    pub edges: HashSet<(ProducerIndex<NI>, CI)>,
}

impl<N, NI, CI> Graph<N, NI, CI>
where
    N: NodeClass<Class = NI::Class>,
    NI: NodeIndex,
    CI: ConsumerIndexT<NodeIndex = NI>,
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
            producer.node_index != node_index && consumer.node_index() != node_index
        });
    }

    pub fn node(&self, node_index: &NI) -> &N {
        self.nodes
            .get(node_index)
            .expect("The node for the given index was not found")
    }

    pub fn node_mut(&mut self, node_index: &NI) -> &mut N {
        self.nodes
            .get_mut(node_index)
            .expect("The node for the given index was not found")
    }

    pub fn add_edge(&mut self, producer: ProducerIndex<NI>, consumer: CI) {
        self.edges
            .iter()
            .for_each(|(existing_producer, existing_consumer)| {
                if *existing_consumer == consumer && *existing_producer != producer {
                    panic!("Each consumer must be connected to the maximum of a single producer at the time");
                }
            });
        self.edges.insert((producer, consumer));
    }

    pub fn remove_edge(&mut self, producer: ProducerIndex<NI>, consumer: CI) {
        self.edges.remove(&(producer, consumer));
    }

    pub fn has_edge(&mut self, producer: ProducerIndex<NI>, consumer: CI) -> bool {
        self.edges.contains(&(producer, consumer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

    type TestConsumerIndex = ConsumerIndex<TestNodeIndex>;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestProducer;

    type TestProducerIndex = ProducerIndex<TestNodeIndex>;

    impl NodeIndex for TestNodeIndex {
        type Class = TestClass;
        type Consumer = TestConsumer;
        type ConsumerIndex = TestConsumerIndex;
        type Producer = TestProducer;

        fn new(_class: TestClass, index: usize) -> Self {
            Self { index }
        }

        fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
        where
            IntoC: Into<Self::Consumer>,
        {
            ConsumerIndex::new(*self, consumer.into())
        }

        fn producer<IntoP>(&self, producer: IntoP) -> TestProducerIndex
        where
            IntoP: Into<TestProducer>,
        {
            ProducerIndex::new(*self, producer.into())
        }
    }

    type TestGraph = Graph<TestNode, TestNodeIndex, TestConsumerIndex>;

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

        assert_eq!(consumer_index.node_index, node_index)
    }

    #[test]
    fn get_consumer_index_consumer() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let consumer_index = node_index.consumer(TestConsumer);

        assert_eq!(consumer_index.consumer, TestConsumer)
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

        assert_eq!(producer_index.node_index, node_index)
    }

    #[test]
    fn get_producer_index_producer() {
        let node_index = TestNodeIndex::new(TestClass, 0);
        let producer_index = node_index.producer(TestProducer);

        assert_eq!(producer_index.producer, TestProducer)
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
    #[should_panic(expected = "The node for the given index was not found")]
    fn remove_node() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        graph.remove_node(index);

        graph.node(&index);
    }

    #[test]
    fn get_node() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        assert_eq!(*graph.node(&index), TestNode(10));
    }

    #[test]
    #[should_panic(expected = "The node for the given index was not found")]
    fn panic_on_get_nonexistent_node() {
        let graph = TestGraph::new();

        graph.node(&NodeIndex::new(TestClass, 100));
    }

    #[test]
    fn get_node_mut() {
        let mut graph = TestGraph::new();
        let index = graph.add_node(10);

        *graph.node_mut(&index) = 20.into();
        assert_eq!(*graph.node(&index), 20.into());
    }

    #[test]
    #[should_panic(expected = "The node for the given index was not found")]
    fn panic_on_get_nonexistent_node_mut() {
        let mut graph = TestGraph::new();

        graph.node_mut(&NodeIndex::new(TestClass, 100));
    }

    #[test]
    fn add_edge() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));

        graph.add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        assert!(graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn add_multiple_edges_with_single_source() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);

        graph.add_edge(one.producer(TestProducer), two.consumer(TestConsumer));
        graph.add_edge(one.producer(TestProducer), three.consumer(TestConsumer));

        assert!(graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
        assert!(graph.has_edge(one.producer(TestProducer), three.consumer(TestConsumer)));
    }

    #[test]
    #[should_panic(
        expected = "Each consumer must be connected to the maximum of a single producer at the time"
    )]
    fn panic_on_add_multiple_edges_with_single_destination() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);

        graph.add_edge(one.producer(TestProducer), three.consumer(TestConsumer));
        graph.add_edge(two.producer(TestProducer), three.consumer(TestConsumer));
    }

    #[test]
    fn remove_edge() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        graph.add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        graph.remove_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn remove_edges_on_source_node_removal() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        graph.add_edge(one.producer(TestProducer), two.consumer(TestConsumer));

        graph.remove_node(one);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
    }

    #[test]
    fn remove_edges_on_destination_node_removal() {
        let mut graph = TestGraph::new();
        let one = graph.add_node(1);
        let two = graph.add_node(2);
        let three = graph.add_node(3);
        graph.add_edge(one.producer(TestProducer), two.consumer(TestConsumer));
        graph.add_edge(one.producer(TestProducer), three.consumer(TestConsumer));

        graph.remove_node(two);

        assert!(!graph.has_edge(one.producer(TestProducer), two.consumer(TestConsumer)));
        assert!(graph.has_edge(one.producer(TestProducer), three.consumer(TestConsumer)));
    }
}
