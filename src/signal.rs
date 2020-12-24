// TODO: Implemented over graph trait
// TODO: Would apply feedback nodes if needed
// TODO: Would know how to traverse the graph

use std::collections::HashSet;
use std::hash::Hash;

use crate::graph::{ConsumerIndex, Graph, NodeIndex, ProducerIndex};
use crate::internal::{
    InternalNode, InternalNodeClass, InternalNodeIndex, InternalNodeInput, InternalNodeOutput,
};
use crate::node::{ExternalConsumer, ExternalNodeWrapper, ExternalProducer, Node, NodeWrapper};
use crate::sort;

// TODO XXX The node index must be implemented on signalnode level

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeIndex<NI> {
    RegisteredNode(NI),
    InternalNode(InternalNodeIndex),
}

pub enum SignalNodeClass<NC> {
    RegisteredNode(NC),
    InternalNode(InternalNodeClass),
}

pub type SignalNodeInputIndex<NI, C, P> =
    ConsumerIndex<SignalNodeIndex<NI>, SignalNodeInput<C>, SignalNodeOutput<P>>;
pub type SignalNodeOutputIndex<NI, C, P> =
    ProducerIndex<SignalNodeIndex<NI>, SignalNodeInput<C>, SignalNodeOutput<P>>;

impl<NI, C, P> NodeIndex<SignalNodeInput<C>, SignalNodeOutput<P>> for SignalNodeIndex<NI>
where
    NI: NodeIndex<C, P>,
    C: Copy + Hash,
    P: Copy + Hash,
{
    fn new(_index: usize) -> Self {
        panic!("Do not use this");
    }

    fn consumer<IntoC>(&self, consumer: IntoC) -> SignalNodeInputIndex<NI, C, P>
    where
        IntoC: Into<SignalNodeInput<C>>,
    {
        ConsumerIndex::new(*self, consumer.into())
    }

    fn producer<IntoP>(&self, producer: IntoP) -> SignalNodeOutputIndex<NI, C, P>
    where
        IntoP: Into<SignalNodeOutput<P>>,
    {
        ProducerIndex::new(*self, producer.into())
    }
}

pub enum SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    RegisteredNode(N),
    InternalNode(InternalNode<i32>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeInput<C>
where
    C: Copy + Hash,
{
    RegisteredNode(C),
    InternalNode(InternalNodeInput),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeOutput<P>
where
    P: Copy + Hash,
{
    RegisteredNode(P),
    InternalNode(InternalNodeOutput),
}

// TODO: Is this used?
impl<N> From<N> for SignalNode<N>
where
    N: ExternalNodeWrapper<i32>,
{
    fn from(node: N) -> Self {
        Self::RegisteredNode(node.into())
    }
}

// TODO: Is this used?
impl<N> From<InternalNode<i32>> for SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    fn from(node: InternalNode<i32>) -> Self {
        Self::InternalNode(node)
    }
}

impl<C> From<InternalNodeInput> for SignalNodeInput<C>
where
    C: Copy + Hash,
{
    fn from(consumer: InternalNodeInput) -> Self {
        Self::InternalNode(consumer.into())
    }
}

impl<P> From<InternalNodeOutput> for SignalNodeOutput<P>
where
    P: Copy + Hash,
{
    fn from(producer: InternalNodeOutput) -> Self {
        Self::InternalNode(producer.into())
    }
}

impl<C> From<C> for SignalNodeInput<C>
where
    C: ExternalConsumer,
{
    fn from(consumer: C) -> Self {
        Self::RegisteredNode(consumer.into())
    }
}

impl<P> From<P> for SignalNodeOutput<P>
where
    P: ExternalProducer,
{
    fn from(producer: P) -> Self {
        Self::RegisteredNode(producer.into())
    }
}

// TODO How to differentiate between user provided and internal?!

impl<N> NodeWrapper<i32> for SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    type Consumer = SignalNodeInput<N::Consumer>;
    type Producer = SignalNodeOutput<N::Producer>;
    type Class = SignalNodeClass<N::Class>;

    fn class(&self) -> Self::Class {
        match self {
            Self::RegisteredNode(registered_node) => {
                Self::Class::RegisteredNode(registered_node.class())
            }
            Self::InternalNode(internal_node) => Self::Class::InternalNode(internal_node.class()),
        }
    }

    fn tick(&mut self) {
        match self {
            Self::RegisteredNode(registered_node) => registered_node.tick(),
            Self::InternalNode(internal_node) => internal_node.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> i32
    where
        IntoP: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
            Self::RegisteredNode(registered_node) => match producer {
                Self::Producer::RegisteredNode(producer) => registered_node.read(producer),
                _ => panic!("Bad"),
            },
            Self::InternalNode(internal_node) => match producer {
                Self::Producer::InternalNode(producer) => internal_node.read(producer),
                _ => panic!("Bad"),
            },
        }
    }

    fn write<IntoC>(&mut self, consumer: IntoC, input: i32)
    where
        IntoC: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
            Self::RegisteredNode(registered_node) => match consumer {
                Self::Consumer::RegisteredNode(consumer) => registered_node.write(consumer, input),
                _ => panic!("Bad"),
            },
            Self::InternalNode(internal_node) => match consumer {
                Self::Consumer::InternalNode(consumer) => internal_node.write(consumer, input),
                _ => panic!("Bad"),
            },
        }
    }
}

// TODO XXX Implement Node over SignalNode

struct SignalGraph<N, NI, C, P>
where
    N: NodeWrapper<i32>,
{
    graph: Graph<N, NI, C, P>,
}

impl<N, NI, C, P> SignalGraph<N, NI, C, P>
where
    N: NodeWrapper<i32>,
    NI: NodeIndex<C, P>,
    C: Copy + Eq + Hash,
    P: Copy + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
        }
    }

    pub fn add_node<IntoN>(&mut self, node: IntoN) -> NI
    where
        IntoN: Into<N>,
    {
        self.graph.add_node(node.into())
    }

    pub fn remove_node(&mut self, node_index: NI) {
        self.graph.remove_node(node_index)
    }

    pub fn node(&self, node_index: &NI) -> &N {
        self.graph.node(node_index)
    }

    pub fn node_mut(&mut self, node_index: &NI) -> &mut N {
        self.graph.node_mut(node_index)
    }

    pub fn add_edge(
        &mut self,
        producer: ProducerIndex<NI, C, P>,
        consumer: ConsumerIndex<NI, C, P>,
    ) {
        self.graph.add_edge(producer, consumer)
    }

    pub fn remove_edge(
        &mut self,
        producer: ProducerIndex<NI, C, P>,
        consumer: ConsumerIndex<NI, C, P>,
    ) {
        self.graph.remove_edge(producer, consumer)
    }

    pub fn has_edge(
        &mut self,
        producer: ProducerIndex<NI, C, P>,
        consumer: ConsumerIndex<NI, C, P>,
    ) -> bool {
        self.graph.has_edge(producer, consumer)
    }

    pub fn tick(&mut self) {
        // TODO: Calculate the path and use it to traverse the graph
        // TODO: Keep this path in cache

        // TODO: Improve this
        let nodes: HashSet<_> = self.graph.nodes.keys().copied().collect();
        let edges: HashSet<(_, _)> = {
            let mut set = HashSet::new();
            for (source_index, destination_indexes) in self.graph.edges.iter() {
                for destination_index in destination_indexes.iter() {
                    set.insert((source_index.node_index, destination_index.node_index));
                }
            }
            set
        };

        let sorted_nodes = match sort::topological_sort(nodes, edges) {
            Ok(nodes) => nodes,
            _ => panic!("Bad"),
        };

        // TODO: Define a function on graph that would allow us to iterate all node pairs
        // TODO: Make this more efficient
        for node_index in sorted_nodes.iter() {
            self.graph.nodes.get_mut(node_index).unwrap().tick();

            for (source_index, destination_indexes) in self.graph.edges.iter() {
                if source_index.node_index != *node_index {
                    continue;
                }

                for destination_index in destination_indexes.iter() {
                    // let source = self.graph.node(&source_index.node_index);
                    // let output = source.read(source_index.producer);
                    // let destination = self
                    //     .graph
                    //     .nodes
                    //     .get_mut(&destination_index.node_index)
                    //     .unwrap();
                    // destination.write(destination_index.consumer, output);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::{self, FeedbackSinkOutput, FeedbackSourceInput};
    use crate::node::{Node, NodeWrapper};

    // TODO: Can we drop the Eq requirement?
    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct Number(i32);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum NumberInput {}

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct NumberOutput;

    impl Node<i32> for Number {
        type Consumer = NumberInput;
        type Producer = NumberOutput;

        fn read(&self, _producer: Self::Producer) -> i32 {
            self.0
        }
    }

    impl From<Number> for TestNode {
        fn from(number: Number) -> Self {
            Self::Number(number)
        }
    }

    impl From<NumberInput> for TestConsumer {
        fn from(number: NumberInput) -> Self {
            Self::Number(number)
        }
    }

    impl From<NumberOutput> for TestProducer {
        fn from(number: NumberOutput) -> Self {
            Self::Number(number)
        }
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Default)]
    struct Plus {
        input1: i32,
        input2: i32,
        output: i32,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum PlusInput {
        In1,
        In2,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct PlusOutput;

    impl Node<i32> for Plus {
        type Consumer = PlusInput;
        type Producer = PlusOutput;

        fn tick(&mut self) {
            self.output = self.input1 + self.input2;
        }

        fn read(&self, _producer: Self::Producer) -> i32 {
            self.output
        }

        fn write(&mut self, consumer: Self::Consumer, input: i32) {
            match consumer {
                Self::Consumer::In1 => self.input1 = input,
                Self::Consumer::In2 => self.input2 = input,
            }
        }
    }

    impl From<Plus> for TestNode {
        fn from(plus: Plus) -> Self {
            Self::Plus(plus)
        }
    }

    impl From<PlusInput> for TestConsumer {
        fn from(plus: PlusInput) -> Self {
            Self::Plus(plus)
        }
    }

    impl From<PlusOutput> for TestProducer {
        fn from(plus: PlusOutput) -> Self {
            Self::Plus(plus)
        }
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Default)]
    struct Recorder(i32);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderInput;

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderOutput;

    impl Node<i32> for Recorder {
        type Consumer = RecorderInput;
        type Producer = RecorderOutput;

        fn read(&self, _producer: Self::Producer) -> i32 {
            self.0
        }

        fn write(&mut self, _consumer: Self::Consumer, input: i32) {
            self.0 = input;
        }
    }

    impl From<Recorder> for TestNode {
        fn from(recorder: Recorder) -> Self {
            Self::Recorder(recorder)
        }
    }

    impl From<RecorderInput> for TestConsumer {
        fn from(recorder: RecorderInput) -> Self {
            Self::Recorder(recorder)
        }
    }

    impl From<RecorderOutput> for TestProducer {
        fn from(recorder: RecorderOutput) -> Self {
            Self::Recorder(recorder)
        }
    }

    #[derive(PartialEq, Eq)]
    enum TestNode {
        Number(Number),
        Plus(Plus),
        Recorder(Recorder),
    }

    enum TestNodeClass {
        Number,
        Plus,
        Recorder,
    }

    impl NodeWrapper<i32> for TestNode {
        type Consumer = TestConsumer;
        type Producer = TestProducer;
        type Class = TestNodeClass;

        fn class(&self) -> Self::Class {
            match self {
                Self::Number(_) => TestNodeClass::Number,
                Self::Plus(_) => TestNodeClass::Plus,
                Self::Recorder(_) => TestNodeClass::Recorder,
            }
        }

        fn tick(&mut self) {
            match self {
                Self::Number(number) => number.tick(),
                Self::Plus(plus) => plus.tick(),
                Self::Recorder(recorder) => recorder.tick(),
            }
        }

        fn read<IntoP>(&self, producer: IntoP) -> i32
        where
            IntoP: Into<Self::Producer>,
        {
            let producer = producer.into();
            match self {
                Self::Number(number) => match producer {
                    Self::Producer::Number(producer) => number.read(producer),
                    _ => panic!("Bad bad, not good"),
                },
                Self::Plus(plus) => match producer {
                    Self::Producer::Plus(producer) => plus.read(producer),
                    _ => panic!("Bad bad, not good"),
                },
                Self::Recorder(recorder) => match producer {
                    Self::Producer::Recorder(producer) => recorder.read(producer),
                    _ => panic!("Bad bad, not good"),
                },
            }
        }

        fn write<IntoC>(&mut self, consumer: IntoC, input: i32)
        where
            IntoC: Into<Self::Consumer>,
        {
            let consumer = consumer.into();
            dbg!(consumer);
            match self {
                Self::Number(number) => match consumer {
                    Self::Consumer::Number(consumer) => number.write(consumer.into(), input),
                    _ => panic!("Bad bad, not good"),
                },
                Self::Plus(plus) => match consumer {
                    Self::Consumer::Plus(consumer) => plus.write(consumer.into(), input),
                    _ => panic!("Bad bad, not good"),
                },
                Self::Recorder(recorder) => match consumer {
                    Self::Consumer::Recorder(consumer) => recorder.write(consumer.into(), input),
                    _ => panic!("Bad bad, not good"),
                },
            }
        }
    }

    impl ExternalNodeWrapper<i32> for TestNode {}

    // TODO: Can we move this and its implementation to a lib too?
    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct TestNodeIndex {
        index: usize,
        // TODO: Keep the Node class too, so we can verify that the consumer belongs to it
    }

    impl NodeIndex<TestConsumer, TestProducer> for TestNodeIndex {
        fn new(index: usize) -> Self {
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

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum TestConsumer {
        Number(NumberInput),
        Plus(PlusInput),
        Recorder(RecorderInput),
    }

    impl ExternalConsumer for TestConsumer {}

    type TestConsumerIndex = ConsumerIndex<TestNodeIndex, TestConsumer, TestProducer>;

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum TestProducer {
        Number(NumberOutput),
        Plus(PlusOutput),
        Recorder(RecorderOutput),
    }

    impl ExternalProducer for TestProducer {}

    type TestProducerIndex = ProducerIndex<TestNodeIndex, TestConsumer, TestProducer>;

    // type TestSignalGraph = SignalGraph<TestNode, TestNodeIndex, TestConsumer, TestProducer>;
    // TODO: Wrap it
    type TestSignalGraph = SignalGraph<
        SignalNode<TestNode>,
        SignalNodeIndex<TestNodeIndex>,
        SignalNodeInput<TestConsumer>,
        SignalNodeOutput<TestProducer>,
    >;

    fn new_node<N, IntoN>(node: IntoN) -> SignalNode<N>
    where
        IntoN: Into<N>,
        N: NodeWrapper<i32>,
    {
        SignalNode::RegisteredNode(node.into())
    }

    fn new_consumer<C, IntoC>(consumer: IntoC) -> SignalNodeInput<C>
    where
        IntoC: Into<C>,
        C: Copy + Hash,
    {
        SignalNodeInput::RegisteredNode(consumer.into())
    }

    fn new_producer<P, IntoP>(producer: IntoP) -> SignalNodeOutput<P>
    where
        IntoP: Into<P>,
        P: Copy + Hash,
    {
        SignalNodeOutput::RegisteredNode(producer.into())
    }

    #[test]
    fn convert_internal_node_to_signal_node() {
        let (source, _sink) = feedback::new_feedback_pair();

        let _node: SignalNode<TestNode> = SignalNode::InternalNode(source.into());
    }

    #[test]
    fn convert_registered_node_to_signal_node() {
        let _node: SignalNode<TestNode> = SignalNode::RegisteredNode(Number(10).into());
    }

    #[test]
    fn write_tick_read_internal_signal_node() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: SignalNode<TestNode> = SignalNode::InternalNode(source.into());
        let mut sink: SignalNode<TestNode> = SignalNode::InternalNode(sink.into());

        // TODO: This would be ideally without wrapping
        source.write(InternalNodeInput::FeedbackSource(FeedbackSourceInput), 10);
        assert_eq!(
            sink.read(InternalNodeOutput::FeedbackSink(FeedbackSinkOutput)),
            10
        );
    }

    #[test]
    fn write_tick_read_registered_signal_node() {
        let mut node: SignalNode<TestNode> = SignalNode::RegisteredNode(Plus::default().into());

        // TODO: This would be ideally without wrapping
        node.write(TestConsumer::Plus(PlusInput::In1), 10);
        node.write(TestConsumer::Plus(PlusInput::In2), 20);
        node.tick();
        assert_eq!(node.read(TestProducer::Plus(PlusOutput)), 30);
    }

    // Simple tree:
    //
    //    [Rec]
    //      |
    //     [+]
    //    /   \
    //  [1]   [2]
    //
    // Should save 3 to the end consumer.
    #[test]
    fn simple_tree() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(new_node(Number(1)));
        let two = graph.add_node(new_node(Number(2)));
        let plus = graph.add_node(new_node(Plus::default()));
        let recorder = graph.add_node(new_node(Recorder::default()));
        graph.add_edge(
            one.producer(new_producer(NumberOutput)),
            plus.consumer(new_consumer(PlusInput::In1)),
        );
        graph.add_edge(
            two.producer(new_producer(NumberOutput)),
            plus.consumer(new_consumer(PlusInput::In2)),
        );
        graph.add_edge(
            plus.producer(new_producer(PlusOutput)),
            recorder.consumer(new_consumer(RecorderInput)),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder).read(new_producer(RecorderOutput)), 3);
    }

    // Graph with 2 end consumers:
    //
    //  [Rec]   [Rec]
    //      \   /
    //       [+]
    //      /   \
    //    [1]   [2]
    //
    // Should save 3 to both.
    #[test]
    fn multiple_consumers() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(new_node(Number(1)));
        let two = graph.add_node(new_node(Number(2)));
        let plus = graph.add_node(new_node(Plus::default()));
        let recorder1 = graph.add_node(new_node(Recorder::default()));
        let recorder2 = graph.add_node(new_node(Recorder::default()));
        graph.add_edge(
            one.producer(new_producer(NumberOutput)),
            plus.consumer(new_consumer(PlusInput::In1)),
        );
        graph.add_edge(
            two.producer(new_producer(NumberOutput)),
            plus.consumer(new_consumer(PlusInput::In2)),
        );
        graph.add_edge(
            plus.producer(new_producer(PlusOutput)),
            recorder1.consumer(new_consumer(RecorderInput)),
        );
        graph.add_edge(
            plus.producer(new_producer(PlusOutput)),
            recorder2.consumer(new_consumer(RecorderInput)),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder2).read(new_producer(RecorderOutput)), 3);
        assert_eq!(graph.node(&recorder2).read(new_producer(RecorderOutput)), 3);
    }

    // Graph with a loop:
    //
    //  [Rec]    __
    //      \   /  |
    //       [+]   V
    //      /   \__|
    //    [1]
    //
    // Should feedback and keep increasing the recorded value.
    #[test]
    fn internal_cycle() {
        //panic!("Not implemented");
    }
}
