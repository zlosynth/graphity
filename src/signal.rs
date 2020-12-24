// TODO: Implemented over graph trait

use std::collections::{HashMap, HashSet};
use std::hash::Hash;

use crate::feedback::{
    self, FeedbackSink, FeedbackSinkOutput, FeedbackSource, FeedbackSourceInput,
};
use crate::graph::{ConsumerIndex, Graph, NodeIndex, ProducerIndex};
use crate::internal::{
    InternalNode, InternalNodeClass, InternalNodeIndex, InternalNodeInput, InternalNodeOutput,
};
use crate::node::{
    ExternalConsumer, ExternalNodeWrapper, ExternalProducer, NodeClass, NodeWrapper,
};
use crate::sort;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeIndex<NI> {
    RegisteredNode(NI),
    InternalNode(InternalNodeIndex),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeClass<NC> {
    RegisteredNode(NC),
    InternalNode(InternalNodeClass),
}

pub type SignalNodeInputIndex<NI> = ConsumerIndex<SignalNodeIndex<NI>>;
pub type SignalNodeOutputIndex<NI> = ProducerIndex<SignalNodeIndex<NI>>;

impl<NI> NodeIndex for SignalNodeIndex<NI>
where
    NI: NodeIndex,
{
    type Class = SignalNodeClass<NI::Class>;
    type Consumer = SignalNodeInput<NI::Consumer>;
    type Producer = SignalNodeOutput<NI::Producer>;

    fn new(class: SignalNodeClass<NI::Class>, index: usize) -> Self {
        match class {
            SignalNodeClass::RegisteredNode(class) => Self::RegisteredNode(NI::new(class, index)),
            SignalNodeClass::InternalNode(class) => {
                Self::InternalNode(InternalNodeIndex::new(class, index))
            }
        }
    }

    fn consumer<IntoC>(&self, consumer: IntoC) -> SignalNodeInputIndex<NI>
    where
        IntoC: Into<SignalNodeInput<NI::Consumer>>,
    {
        ConsumerIndex::new(*self, consumer.into())
    }

    fn producer<IntoP>(&self, producer: IntoP) -> SignalNodeOutputIndex<NI>
    where
        IntoP: Into<SignalNodeOutput<NI::Producer>>,
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

// TODO XXX Implement From for all stuff than can be into'd into N

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

impl<N> From<FeedbackSource<i32>> for SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    fn from(feedback_source: FeedbackSource<i32>) -> Self {
        Self::InternalNode(InternalNode::FeedbackSource(feedback_source))
    }
}

impl<N> From<FeedbackSink<i32>> for SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    fn from(feedback_sink: FeedbackSink<i32>) -> Self {
        Self::InternalNode(InternalNode::FeedbackSink(feedback_sink))
    }
}

impl<C> From<FeedbackSourceInput> for SignalNodeInput<C>
where
    C: Hash + Copy,
{
    fn from(feedback_source: FeedbackSourceInput) -> Self {
        Self::InternalNode(InternalNodeInput::FeedbackSource(feedback_source))
    }
}

impl<P> From<FeedbackSinkOutput> for SignalNodeOutput<P>
where
    P: Hash + Copy,
{
    fn from(feedback_sink: FeedbackSinkOutput) -> Self {
        Self::InternalNode(InternalNodeOutput::FeedbackSink(feedback_sink))
    }
}

impl<N> NodeClass for SignalNode<N>
where
    N: NodeClass + NodeWrapper<i32>,
{
    type Class = SignalNodeClass<N::Class>;

    fn class(&self) -> Self::Class {
        match self {
            Self::RegisteredNode(registered_node) => {
                Self::Class::RegisteredNode(registered_node.class())
            }
            Self::InternalNode(internal_node) => Self::Class::InternalNode(internal_node.class()),
        }
    }
}

impl<N> NodeWrapper<i32> for SignalNode<N>
where
    N: NodeWrapper<i32>,
{
    type Consumer = SignalNodeInput<N::Consumer>;
    type Producer = SignalNodeOutput<N::Producer>;

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

struct SignalGraph<N, NI>
where
    N: NodeWrapper<i32, Class = NI::Class>,
    NI: NodeIndex,
    <N as NodeWrapper<i32>>::Producer: std::convert::From<NI::Producer>,
    <N as NodeWrapper<i32>>::Consumer: std::convert::From<NI::Consumer>,
{
    graph: Graph<i32, N, NI>,
    feedback_edges: HashMap<(ProducerIndex<NI>, ConsumerIndex<NI>), (NI, NI)>,
}

impl<N, NI> SignalGraph<N, NI>
where
    N: NodeWrapper<i32, Class = NI::Class> + std::convert::From<InternalNode<i32>>,
    NI: NodeIndex,
    NI::Consumer: Copy + Eq + Hash + std::convert::From<InternalNodeInput>,
    NI::Producer: Copy + Eq + Hash + std::convert::From<InternalNodeOutput>,
    <N as NodeWrapper<i32>>::Producer: std::convert::From<NI::Producer>,
    <N as NodeWrapper<i32>>::Consumer: std::convert::From<NI::Consumer>,
{
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            feedback_edges: HashMap::new(),
        }
    }

    pub fn add_node<IntoN>(&mut self, node: IntoN) -> NI
    where
        IntoN: Into<N>,
    {
        self.graph.add_node(node.into())
    }

    pub fn remove_node(&mut self, node_index: NI) {
        self.graph.remove_node(node_index);
    }

    pub fn node(&self, node_index: &NI) -> &N {
        self.graph.node(node_index)
    }

    pub fn node_mut(&mut self, node_index: &NI) -> &mut N {
        self.graph.node_mut(node_index)
    }

    pub fn add_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) {
        if self.has_edge(producer, consumer) {
            return;
        }

        self.graph.add_edge(producer, consumer);

        // TODO cleanup
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
        if let Err(Cycle) = sort::topological_sort(nodes, edges) {
            self.graph.remove_edge(producer, consumer);
            let (feedback_source, feedback_sink) = feedback::new_feedback_pair::<i32>();
            let feedback_source = self
                .graph
                .add_node(InternalNode::FeedbackSource(feedback_source));
            let feedback_sink = self
                .graph
                .add_node(InternalNode::FeedbackSink(feedback_sink));
            self.graph.add_edge(
                producer,
                feedback_source.consumer(InternalNodeInput::FeedbackSource(FeedbackSourceInput)),
            );
            self.graph.add_edge(
                feedback_sink.producer(InternalNodeOutput::FeedbackSink(FeedbackSinkOutput)),
                consumer,
            );
            self.feedback_edges
                .insert((producer, consumer), (feedback_source, feedback_sink));
        }
    }

    pub fn remove_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) {
        if self.graph.has_edge(producer, consumer) {
            self.graph.remove_edge(producer, consumer);
        } else if self.feedback_edges.contains_key(&(producer, consumer)) {
            let (source_index, sink_index) =
                self.feedback_edges.get(&(producer, consumer)).unwrap();
            self.graph.remove_node(*source_index);
            self.graph.remove_node(*sink_index);
            self.feedback_edges.remove(&(producer, consumer));
        }

        // TODO: Remove as many feedbacks as possible
        // TODO: Make this pretty, this is horrendus
        {
            let mut edges_to_remove = HashSet::new();
            for ((producer, consumer), (feedback_source_index, feedback_sink_index)) in
                self.feedback_edges.iter()
            {
                self.graph.add_edge(*producer, *consumer);

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

                let has_cycle = match sort::topological_sort(nodes, edges) {
                    Err(Cycle) => true,
                    Ok(_) => false,
                };

                if has_cycle {
                    self.graph.remove_edge(*producer, *consumer);
                } else {
                    self.graph.remove_node(*feedback_source_index);
                    self.graph.remove_node(*feedback_sink_index);
                    edges_to_remove.insert((*producer, *consumer));
                }
            }
            for (producer, consumer) in edges_to_remove.iter() {
                self.feedback_edges.remove(&(*producer, *consumer));
            }
        }
    }

    pub fn has_edge(&mut self, producer: ProducerIndex<NI>, consumer: ConsumerIndex<NI>) -> bool {
        // TODO: Consider feedbacks too
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
                    let source = self.graph.node(&source_index.node_index);
                    let output = source.read(source_index.producer);
                    let destination = self
                        .graph
                        .nodes
                        .get_mut(&destination_index.node_index)
                        .unwrap();
                    destination.write(destination_index.consumer, output);
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

    impl From<Number> for SignalNode<TestNode> {
        fn from(number: Number) -> Self {
            Self::RegisteredNode(TestNode::Number(number))
        }
    }

    impl From<NumberInput> for SignalNodeInput<TestConsumer> {
        fn from(number: NumberInput) -> Self {
            Self::RegisteredNode(TestConsumer::Number(number))
        }
    }

    impl From<NumberOutput> for SignalNodeOutput<TestProducer> {
        fn from(number: NumberOutput) -> Self {
            Self::RegisteredNode(TestProducer::Number(number))
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

    impl From<Plus> for SignalNode<TestNode> {
        fn from(plus: Plus) -> Self {
            Self::RegisteredNode(TestNode::Plus(plus))
        }
    }

    impl From<PlusInput> for SignalNodeInput<TestConsumer> {
        fn from(plus: PlusInput) -> Self {
            Self::RegisteredNode(TestConsumer::Plus(plus))
        }
    }

    impl From<PlusOutput> for SignalNodeOutput<TestProducer> {
        fn from(plus: PlusOutput) -> Self {
            Self::RegisteredNode(TestProducer::Plus(plus))
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

    impl From<Recorder> for SignalNode<TestNode> {
        fn from(recorder: Recorder) -> Self {
            Self::RegisteredNode(TestNode::Recorder(recorder))
        }
    }

    impl From<RecorderInput> for SignalNodeInput<TestConsumer> {
        fn from(recorder: RecorderInput) -> Self {
            Self::RegisteredNode(TestConsumer::Recorder(recorder))
        }
    }

    impl From<RecorderOutput> for SignalNodeOutput<TestProducer> {
        fn from(recorder: RecorderOutput) -> Self {
            Self::RegisteredNode(TestProducer::Recorder(recorder.into()))
        }
    }

    #[derive(PartialEq, Eq)]
    enum TestNode {
        Number(Number),
        Plus(Plus),
        Recorder(Recorder),
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum TestNodeClass {
        Number,
        Plus,
        Recorder,
    }

    impl NodeClass for TestNode {
        type Class = TestNodeClass;

        fn class(&self) -> Self::Class {
            match self {
                Self::Number(_) => TestNodeClass::Number,
                Self::Plus(_) => TestNodeClass::Plus,
                Self::Recorder(_) => TestNodeClass::Recorder,
            }
        }
    }

    impl NodeWrapper<i32> for TestNode {
        type Consumer = TestConsumer;
        type Producer = TestProducer;

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

    impl NodeIndex for TestNodeIndex {
        type Class = TestNodeClass;
        type Consumer = TestConsumer;
        type Producer = TestProducer;

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

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum TestConsumer {
        Number(NumberInput),
        Plus(PlusInput),
        Recorder(RecorderInput),
    }

    impl ExternalConsumer for TestConsumer {}

    type TestConsumerIndex = ConsumerIndex<TestNodeIndex>;

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum TestProducer {
        Number(NumberOutput),
        Plus(PlusOutput),
        Recorder(RecorderOutput),
    }

    impl ExternalProducer for TestProducer {}

    type TestProducerIndex = ProducerIndex<TestNodeIndex>;

    type TestSignalGraph = SignalGraph<SignalNode<TestNode>, SignalNodeIndex<TestNodeIndex>>;

    #[test]
    fn convert_internal_node_to_signal_node() {
        let (source, _sink) = feedback::new_feedback_pair();

        let _node: SignalNode<TestNode> = source.into();
    }

    #[test]
    fn convert_registered_node_to_signal_node() {
        let _node: SignalNode<TestNode> = Number(10).into();
    }

    #[test]
    fn write_tick_read_internal_signal_node() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: SignalNode<TestNode> = source.into();
        let mut sink: SignalNode<TestNode> = sink.into();

        source.write(FeedbackSourceInput, 10);
        source.tick();
        sink.tick();
        assert_eq!(sink.read(FeedbackSinkOutput), 10);
    }

    #[test]
    fn write_tick_read_registered_signal_node() {
        let mut node: SignalNode<TestNode> = Plus::default().into();

        // TODO: This would be ideally without wrapping
        node.write(PlusInput::In1, 10);
        node.write(PlusInput::In2, 20);
        node.tick();
        assert_eq!(node.read(PlusOutput), 30);
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
        let one = graph.add_node(Number(1));
        let two = graph.add_node(Number(2));
        let plus = graph.add_node(Plus::default());
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
        graph.add_edge(two.producer(NumberOutput), plus.consumer(PlusInput::In2));
        graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

        graph.tick();
        // RecorderOutput -> TestProducer -> SignalNodeOutput
        assert_eq!(graph.node(&recorder).read(RecorderOutput), 3);
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
        let one = graph.add_node(Number(1));
        let two = graph.add_node(Number(2));
        let plus = graph.add_node(Plus::default());
        let recorder1 = graph.add_node(Recorder::default());
        let recorder2 = graph.add_node(Recorder::default());
        graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
        graph.add_edge(two.producer(NumberOutput), plus.consumer(PlusInput::In2));
        graph.add_edge(plus.producer(PlusOutput), recorder1.consumer(RecorderInput));
        graph.add_edge(plus.producer(PlusOutput), recorder2.consumer(RecorderInput));

        graph.tick();
        assert_eq!(graph.node(&recorder1).read(RecorderOutput), 3);
        assert_eq!(graph.node(&recorder2).read(RecorderOutput), 3);
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
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
        graph.add_edge(plus.producer(PlusOutput), plus.consumer(PlusInput::In2));
        graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

        graph.tick();
        assert_eq!(graph.node(&recorder).read(RecorderOutput), 1);
        graph.tick();
        assert_eq!(graph.node(&recorder).read(RecorderOutput), 2);
    }

    // TODO: Test that feedback is considered an edge
    // TODO: Test that node removal drops edges
    // TODO: Test all the feedback stuff
    // TODO: Test that feedbacks are removed
}
