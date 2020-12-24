// TODO: Implement iterator for nodes and edges once it is clear which are needed
// TODO: Move all the types to associated types, to clean up
use std::collections::{hash_map, HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;

use crate::node::NodeWrapper;

// TODO: Keep Node class too, to make sure that the consumer/producer is available in the given node
pub trait NodeIndex<NC, C, P>: Copy + Hash + Eq {
    fn new(class: NC, index: usize) -> Self;
    fn consumer<IntoC>(&self, consumer: IntoC) -> ConsumerIndex<NC, Self, C, P>
    where
        IntoC: Into<C>;
    fn producer<IntoP>(&self, producer: IntoP) -> ProducerIndex<NC, Self, C, P>
    where
        IntoP: Into<P>;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ConsumerIndex<NC, NI, C, P> {
    pub node_index: NI,
    pub consumer: C,
    _class: PhantomData<NC>,
    _producer: PhantomData<P>,
}

impl<NI, NC, C, P> ConsumerIndex<NC, NI, C, P>
where
    NI: NodeIndex<NC, C, P>,
    C: Copy + Hash,
{
    pub fn new(node_index: NI, consumer: C) -> Self {
        Self {
            node_index,
            consumer,
            _class: PhantomData,
            _producer: PhantomData,
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct ProducerIndex<NC, NI, C, P> {
    pub node_index: NI,
    pub producer: P,
    _class: PhantomData<NC>,
    _consumer: PhantomData<C>,
}

impl<NI, NC, C, P> ProducerIndex<NC, NI, C, P>
where
    NI: NodeIndex<NC, C, P>,
    P: Copy + Hash,
{
    pub fn new(node_index: NI, producer: P) -> Self {
        Self {
            node_index,
            producer,
            _class: PhantomData,
            _consumer: PhantomData,
        }
    }
}

// TODO: Doc
// Directed graph
// Each node is further divided into producers and consumers
// with MAX 1 indegree for each consumer and arbitrary number of outdegrees for producer
// all consumers of a node are connected to all producers of the node
pub struct Graph<T, N, NC, NI, C, P>
where
    T: Default,
    N: NodeWrapper<T, Class = NC>,
{
    index_counter: usize,
    // TODO: Must make this private
    pub nodes: HashMap<NI, N>,
    // TODO: Must make this private
    // TODO: Turn this to a basic hashset until all usecases are identified
    pub edges: HashMap<ProducerIndex<NC, NI, C, P>, HashSet<ConsumerIndex<NC, NI, C, P>>>,
    _type: PhantomData<T>,
    _class: PhantomData<NC>,
    _consumer: PhantomData<C>,
    _producer: PhantomData<P>,
}

// TODO: Make this into a trait, so it can be implemented by the signal graph too
impl<T, N, NC, NI, C, P> Graph<T, N, NC, NI, C, P>
where
    // TODO: Limit the trait for .class()
    T: Default,
    N: NodeWrapper<T, Class = NC>,
    NI: NodeIndex<NC, C, P>,
    NC: Eq + Hash,
    C: Eq + Hash,
    P: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            index_counter: 0,
            nodes: HashMap::new(),
            edges: HashMap::new(),
            _type: PhantomData,
            _class: PhantomData,
            _consumer: PhantomData,
            _producer: PhantomData,
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
        self.edges.retain(|producer, consumers| {
            if producer.node_index == node_index {
                return false;
            }
            consumers.retain(|consumer| consumer.node_index != node_index);
            true
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

    pub fn nodes(&self) -> hash_map::Values<NI, N> {
        self.nodes.values()
    }

    pub fn nodes_mut(&mut self) -> hash_map::ValuesMut<NI, N> {
        self.nodes.values_mut()
    }

    pub fn add_edge(
        &mut self,
        producer: ProducerIndex<NC, NI, C, P>,
        consumer: ConsumerIndex<NC, NI, C, P>,
    ) {
        self.edges
            .iter()
            .for_each(|(existing_producer, existing_consumers)| {
                if existing_consumers.contains(&consumer) && *existing_producer != producer {
                    panic!("Each consumer must be connected to the maximum of a single producer at the time");
                }
            });
        self.edges
            .entry(producer)
            .or_insert_with(HashSet::new)
            .insert(consumer);
    }

    pub fn remove_edge(
        &mut self,
        producer: ProducerIndex<NC, NI, C, P>,
        consumer: ConsumerIndex<NC, NI, C, P>,
    ) {
        if let Some(consumers) = self.edges.get_mut(&producer) {
            consumers.remove(&consumer);
            if consumers.is_empty() {
                self.edges.remove(&producer);
            }
        }
    }

    pub fn has_edge(
        &mut self,
        producer: ProducerIndex<NC, NI, C, P>,
        consumer: ConsumerIndex<NC, NI, C, P>,
    ) -> bool {
        match self.edges.get(&producer) {
            Some(consumers) => consumers.contains(&consumer),
            None => false,
        }
    }

    // TODO: Define a proper iterator once it is clear whether one is needed
    pub fn edges(
        &self,
    ) -> hash_map::Iter<ProducerIndex<NC, NI, C, P>, HashSet<ConsumerIndex<NC, NI, C, P>>> {
        self.edges.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestNode(i32);

    impl NodeWrapper<i32> for TestNode {
        type Consumer = TestConsumer;
        type Producer = TestProducer;
        type Class = TestNodeClass;

        fn class(&self) -> Self::Class {
            TestNodeClass
        }

        fn tick(&mut self) {
            todo!();
        }

        fn read<IntoP>(&self, producer: IntoP) -> i32
        where
            IntoP: Into<Self::Producer>,
        {
            todo!();
        }

        fn write<IntoC>(&mut self, consumer: IntoC, input: i32)
        where
            IntoC: Into<Self::Consumer>,
        {
            todo!();
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
    struct TestNodeClass;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestConsumer;

    type TestConsumerIndex =
        ConsumerIndex<TestNodeClass, TestNodeIndex, TestConsumer, TestProducer>;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestProducer;

    type TestProducerIndex =
        ProducerIndex<TestNodeClass, TestNodeIndex, TestConsumer, TestProducer>;

    impl NodeIndex<TestNodeClass, TestConsumer, TestProducer> for TestNodeIndex {
        // TODO: Use class?
        fn new(_class: TestNodeClass, index: usize) -> Self {
            Self { index }
        }

        fn consumer<IntoC>(&self, consumer: IntoC) -> TestConsumerIndex
        where
            IntoC: Into<TestConsumer>,
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

    type TestGraph = Graph<i32, TestNode, TestNodeClass, TestNodeIndex, TestConsumer, TestProducer>;

    #[test]
    fn initialize_node_index() {
        let _node_index = TestNodeIndex::new(TestNodeClass, 0);
    }

    #[test]
    fn get_consumer_index() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);

        let _consumer_index = node_index.consumer(TestConsumer);
    }

    #[test]
    fn get_consumer_index_node_index() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);
        let consumer_index = node_index.consumer(TestConsumer);

        assert_eq!(consumer_index.node_index, node_index)
    }

    #[test]
    fn get_consumer_index_consumer() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);
        let consumer_index = node_index.consumer(TestConsumer);

        assert_eq!(consumer_index.consumer, TestConsumer)
    }

    #[test]
    fn get_producer_index() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);

        let _producer_index = node_index.producer(TestProducer);
    }

    #[test]
    fn get_producer_index_node_index() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);
        let producer_index = node_index.producer(TestProducer);

        assert_eq!(producer_index.node_index, node_index)
    }

    #[test]
    fn get_producer_index_producer() {
        let node_index = TestNodeIndex::new(TestNodeClass, 0);
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

        graph.node(&NodeIndex::new(TestNodeClass, 100));
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

        graph.node_mut(&NodeIndex::new(TestNodeClass, 100));
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
