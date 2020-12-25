// TODO: Implement iterator for nodes and edges once it is clear which are needed
// TODO: Move all the types to associated types, to clean up
use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::marker::PhantomData;

// TODO Maybe this should be here instead?
use crate::node::NodeClass;

pub trait NodeIndex: Copy + Hash + Eq {
    type Class: Copy + Hash + Eq;
    type Consumer: Copy + Hash + Eq;
    type Producer: Copy + Hash + Eq;

    fn new(class: Self::Class, index: usize) -> Self;
    fn consumer<IntoC>(&self, consumer: IntoC) -> ConsumerIndex<Self>
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
    pub node_index: NI,
    pub consumer: NI::Consumer,
}

impl<NI> ConsumerIndex<NI>
where
    NI: NodeIndex,
{
    pub fn new(node_index: NI, consumer: NI::Consumer) -> Self {
        Self {
            node_index,
            consumer,
        }
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

// TODO: Doc
// Directed graph
// Each node is further divided into producers and consumers
// with MAX 1 indegree for each consumer and arbitrary number of outdegrees for producer
// all consumers of a node are connected to all producers of the node
pub struct Graph<N, NI>
where
    N: NodeClass<Class = NI::Class>,
    NI: NodeIndex,
{
    index_counter: usize,
    // TODO: Must make this private
    pub nodes: HashMap<NI, N>,
    // TODO: Must make this private
    // TODO: Turn this to a basic hashset until all usecases are identified
    pub edges: HashSet<(ProducerIndex<NI>, ConsumerIndex<NI>)>,
}

// TODO: Make this into a trait, so it can be implemented by the signal graph too
impl<N, NI> Graph<N, NI>
where
    N: NodeClass<Class = NI::Class>,
    NI: NodeIndex,
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
            producer.node_index != node_index && consumer.node_index != node_index
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

    pub fn add_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) {
        self.edges
            .iter()
            .for_each(|(existing_producer, existing_consumer)| {
                if *existing_consumer == consumer && *existing_producer != producer {
                    panic!("Each consumer must be connected to the maximum of a single producer at the time");
                }
            });
        self.edges.insert((producer, consumer));
    }

    pub fn remove_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) {
        self.edges.remove(&(producer, consumer));
    }

    pub fn has_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) -> bool {
        self.edges.contains(&(producer, consumer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestNode(i32);

    impl NodeClass for TestNode {
        type Class = TestNodeClass;

        fn class(&self) -> Self::Class {
            TestNodeClass
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

    type TestConsumerIndex = ConsumerIndex<TestNodeIndex>;

    #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
    struct TestProducer;

    type TestProducerIndex = ProducerIndex<TestNodeIndex>;

    impl NodeIndex for TestNodeIndex {
        type Class = TestNodeClass;
        type Consumer = TestConsumer;
        type Producer = TestProducer;

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

    type TestGraph = Graph<TestNode, TestNodeIndex>;

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
