use std::collections::{HashMap, HashSet};
use std::convert::From;
use std::hash::Hash;

use crate::feedback::{
    self, FeedbackSink, FeedbackSinkProducer, FeedbackSource, FeedbackSourceConsumer,
};
use crate::graph::{ConsumerIndexT, Graph, NodeIndex, ProducerIndexT};
use crate::internal::{
    InternalClass, InternalConsumer, InternalConsumerIndex, InternalNode, InternalNodeIndex,
    InternalProducer, InternalProducerIndex,
};
use crate::node::{NodeClass, NodeWrapper};
use crate::sort;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeIndex<NI> {
    RegisteredNode(NI),
    InternalNode(InternalNodeIndex),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeClass<NC> {
    RegisteredNode(NC),
    InternalNode(InternalClass),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeConsumerIndex<CI>
where
    CI: ConsumerIndexT,
{
    RegisteredNode(CI),
    InternalNode(InternalConsumerIndex),
}

impl<CI> ConsumerIndexT for SignalNodeConsumerIndex<CI>
where
    CI: ConsumerIndexT,
{
    type NodeIndex = SignalNodeIndex<CI::NodeIndex>;

    fn new(
        node_index: Self::NodeIndex,
        consumer: <<Self as ConsumerIndexT>::NodeIndex as NodeIndex>::Consumer,
    ) -> Self {
        match node_index {
            Self::NodeIndex::RegisteredNode(node_index) => match consumer {
                SignalNodeConsumer::RegisteredNode(consumer) => {
                    SignalNodeConsumerIndex::RegisteredNode(CI::new(node_index, consumer))
                }
                _ => panic!("BAD mismatch"),
            },
            Self::NodeIndex::InternalNode(node_index) => match consumer {
                SignalNodeConsumer::InternalNode(consumer) => {
                    SignalNodeConsumerIndex::InternalNode(InternalConsumerIndex::new(
                        node_index, consumer,
                    ))
                }
                _ => panic!("BAD mismatch"),
            },
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        match self {
            Self::RegisteredNode(consumer_index) => {
                SignalNodeIndex::RegisteredNode(consumer_index.node_index())
            }
            Self::InternalNode(consumer_index) => {
                SignalNodeIndex::InternalNode(consumer_index.node_index())
            }
        }
    }

    fn consumer(&self) -> <<Self as ConsumerIndexT>::NodeIndex as NodeIndex>::Consumer {
        match self {
            Self::RegisteredNode(consumer_index) => {
                SignalNodeConsumer::RegisteredNode(consumer_index.consumer())
            }
            Self::InternalNode(consumer_index) => {
                SignalNodeConsumer::InternalNode(consumer_index.consumer())
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeProducerIndex<CI>
where
    CI: ProducerIndexT,
{
    RegisteredNode(CI),
    InternalNode(InternalProducerIndex),
}

impl<CI> ProducerIndexT for SignalNodeProducerIndex<CI>
where
    CI: ProducerIndexT,
{
    type NodeIndex = SignalNodeIndex<CI::NodeIndex>;

    fn new(
        node_index: Self::NodeIndex,
        producer: <<Self as ProducerIndexT>::NodeIndex as NodeIndex>::Producer,
    ) -> Self {
        match node_index {
            Self::NodeIndex::RegisteredNode(node_index) => match producer {
                SignalNodeProducer::RegisteredNode(producer) => {
                    SignalNodeProducerIndex::RegisteredNode(CI::new(node_index, producer))
                }
                _ => panic!("BAD mismatch"),
            },
            Self::NodeIndex::InternalNode(node_index) => match producer {
                SignalNodeProducer::InternalNode(producer) => {
                    SignalNodeProducerIndex::InternalNode(InternalProducerIndex::new(
                        node_index, producer,
                    ))
                }
                _ => panic!("BAD mismatch"),
            },
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        match self {
            Self::RegisteredNode(producer_index) => {
                SignalNodeIndex::RegisteredNode(producer_index.node_index())
            }
            Self::InternalNode(producer_index) => {
                SignalNodeIndex::InternalNode(producer_index.node_index())
            }
        }
    }

    fn producer(&self) -> <<Self as ProducerIndexT>::NodeIndex as NodeIndex>::Producer {
        match self {
            Self::RegisteredNode(producer_index) => {
                SignalNodeProducer::RegisteredNode(producer_index.producer())
            }
            Self::InternalNode(producer_index) => {
                SignalNodeProducer::InternalNode(producer_index.producer())
            }
        }
    }
}

impl<NI> NodeIndex for SignalNodeIndex<NI>
where
    NI: NodeIndex,
{
    type Class = SignalNodeClass<NI::Class>;
    type Consumer = SignalNodeConsumer<NI::Consumer>;
    type ConsumerIndex = SignalNodeConsumerIndex<NI::ConsumerIndex>;
    type Producer = SignalNodeProducer<NI::Producer>;
    type ProducerIndex = SignalNodeProducerIndex<NI::ProducerIndex>;

    fn new(class: SignalNodeClass<NI::Class>, index: usize) -> Self {
        match class {
            Self::Class::RegisteredNode(class) => Self::RegisteredNode(NI::new(class, index)),
            Self::Class::InternalNode(class) => {
                Self::InternalNode(InternalNodeIndex::new(class, index))
            }
        }
    }

    fn consumer<IntoC>(&self, consumer: IntoC) -> SignalNodeConsumerIndex<NI::ConsumerIndex>
    where
        IntoC: Into<Self::Consumer>,
    {
        SignalNodeConsumerIndex::new(*self, consumer.into())
    }

    fn producer<IntoP>(&self, producer: IntoP) -> SignalNodeProducerIndex<NI::ProducerIndex>
    where
        IntoP: Into<Self::Producer>,
    {
        SignalNodeProducerIndex::new(*self, producer.into())
    }
}

pub enum SignalNode<N>
where
    N: NodeWrapper,
{
    RegisteredNode(N),
    InternalNode(InternalNode<N::Payload>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeConsumer<C>
where
    C: Copy + Hash,
{
    RegisteredNode(C),
    InternalNode(InternalConsumer),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalNodeProducer<P>
where
    P: Copy + Hash,
{
    RegisteredNode(P),
    InternalNode(InternalProducer),
}

impl<N> From<FeedbackSource<N::Payload>> for SignalNode<N>
where
    N: NodeWrapper,
{
    fn from(feedback_source: FeedbackSource<N::Payload>) -> Self {
        Self::InternalNode(InternalNode::FeedbackSource(feedback_source))
    }
}

impl<N> From<FeedbackSink<N::Payload>> for SignalNode<N>
where
    N: NodeWrapper,
{
    fn from(feedback_sink: FeedbackSink<N::Payload>) -> Self {
        Self::InternalNode(InternalNode::FeedbackSink(feedback_sink))
    }
}

impl<C> From<FeedbackSourceConsumer> for SignalNodeConsumer<C>
where
    C: Hash + Copy,
{
    fn from(feedback_source: FeedbackSourceConsumer) -> Self {
        Self::InternalNode(InternalConsumer::FeedbackSource(feedback_source))
    }
}

impl<P> From<FeedbackSinkProducer> for SignalNodeProducer<P>
where
    P: Hash + Copy,
{
    fn from(feedback_sink: FeedbackSinkProducer) -> Self {
        Self::InternalNode(InternalProducer::FeedbackSink(feedback_sink))
    }
}

impl<N> NodeClass for SignalNode<N>
where
    N: NodeClass + NodeWrapper,
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

impl<N> NodeWrapper for SignalNode<N>
where
    N: NodeWrapper,
{
    type Payload = N::Payload;
    type Consumer = SignalNodeConsumer<N::Consumer>;
    type Producer = SignalNodeProducer<N::Producer>;

    fn tick(&mut self) {
        match self {
            Self::RegisteredNode(registered_node) => registered_node.tick(),
            Self::InternalNode(internal_node) => internal_node.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> N::Payload
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

    fn write<IntoC>(&mut self, consumer: IntoC, input: N::Payload)
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

struct SignalGraph<N, NI, CI, PI>
where
    N: NodeWrapper<Class = NI::Class>,
    NI: NodeIndex<ConsumerIndex = CI, ProducerIndex = PI>,
    CI: ConsumerIndexT<NodeIndex = NI>,
    PI: ProducerIndexT<NodeIndex = NI>,
{
    graph: Graph<N, NI, CI, PI>,
    feedback_edges: HashMap<(PI, CI), (NI, NI)>,
    sorted_nodes: Vec<NI>,
}

impl<N, NI, CI, PI> SignalGraph<N, NI, CI, PI>
where
    N: NodeWrapper<Class = NI::Class>,
    FeedbackSource<N::Payload>: Into<N>,
    FeedbackSink<N::Payload>: Into<N>,
    <N as NodeWrapper>::Producer: From<NI::Producer>,
    <N as NodeWrapper>::Consumer: From<NI::Consumer>,
    NI: NodeIndex<ConsumerIndex = CI, ProducerIndex = PI>,
    NI::Consumer: From<FeedbackSourceConsumer>,
    NI::Producer: From<FeedbackSinkProducer>,
    CI: ConsumerIndexT<NodeIndex = NI>,
    PI: ProducerIndexT<NodeIndex = NI>,
{
    pub fn new() -> Self {
        Self {
            graph: Graph::new(),
            feedback_edges: HashMap::new(),
            sorted_nodes: Vec::new(),
        }
    }

    pub fn add_node<IntoN>(&mut self, node: IntoN) -> NI
    where
        IntoN: Into<N>,
    {
        let index = self.graph.add_node(node.into());
        self.update_cache();
        index
    }

    pub fn remove_node(&mut self, node_index: NI) {
        self.graph.remove_node(node_index);
        self.update_cache();
    }

    pub fn node(&self, node_index: &NI) -> &N {
        self.graph.node(node_index)
    }

    pub fn node_mut(&mut self, node_index: &NI) -> &mut N {
        self.graph.node_mut(node_index)
    }

    pub fn add_edge(&mut self, producer: PI, consumer: CI) {
        if self.has_edge(producer, consumer) {
            return;
        }

        self.graph.add_edge(producer, consumer);

        // TODO cleanup
        let nodes: HashSet<_> = self.graph.nodes.keys().copied().collect();
        let edges: HashSet<(_, _)> = self
            .graph
            .edges
            .iter()
            .map(|(producer, consumer)| (producer.node_index(), consumer.node_index()))
            .collect();
        if let Err(Cycle) = sort::topological_sort(nodes, edges) {
            self.graph.remove_edge(producer, consumer);
            let (feedback_source, feedback_sink) = feedback::new_feedback_pair::<N::Payload>();
            let feedback_source = self.graph.add_node(feedback_source);
            let feedback_sink = self.graph.add_node(feedback_sink);
            self.graph
                .add_edge(producer, feedback_source.consumer(FeedbackSourceConsumer));
            self.graph
                .add_edge(feedback_sink.producer(FeedbackSinkProducer), consumer);
            self.feedback_edges
                .insert((producer, consumer), (feedback_source, feedback_sink));
        }

        self.update_cache();
    }

    pub fn remove_edge(&mut self, producer: PI, consumer: CI) {
        if self.graph.has_edge(producer, consumer) {
            self.graph.remove_edge(producer, consumer);
        } else if self.feedback_edges.contains_key(&(producer, consumer)) {
            let (source_index, sink_index) =
                self.feedback_edges.get(&(producer, consumer)).unwrap();
            self.graph.remove_node(*source_index);
            self.graph.remove_node(*sink_index);
            self.feedback_edges.remove(&(producer, consumer));
        }

        // TODO: Make this pretty, this is horrendus
        {
            let mut edges_to_remove = HashSet::new();
            for ((producer, consumer), (feedback_source_index, feedback_sink_index)) in
                self.feedback_edges.iter()
            {
                self.graph.add_edge(*producer, *consumer);

                let nodes: HashSet<_> = self.graph.nodes.keys().copied().collect();
                let edges: HashSet<(_, _)> = self
                    .graph
                    .edges
                    .iter()
                    .map(|(producer, consumer)| (producer.node_index(), consumer.node_index()))
                    .collect();

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

        self.update_cache();
    }

    pub fn has_edge(&mut self, producer: PI, consumer: CI) -> bool {
        // TODO: Consider feedbacks too
        self.graph.has_edge(producer, consumer)
    }

    fn update_cache(&mut self) {
        let nodes: HashSet<_> = self.graph.nodes.keys().copied().collect();
        let edges: HashSet<(_, _)> = self
            .graph
            .edges
            .iter()
            .map(|(producer, consumer)| (producer.node_index(), consumer.node_index()))
            .collect();

        let sorted_nodes = match sort::topological_sort(nodes, edges) {
            Ok(nodes) => nodes,
            _ => panic!("Bad"),
        };

        self.sorted_nodes = sorted_nodes;
    }

    pub fn tick(&mut self) {
        // TODO: Define a function on graph that would allow us to iterate all node pairs
        // TODO: Make this more efficient
        for node_index in self.sorted_nodes.iter() {
            self.graph.nodes.get_mut(node_index).unwrap().tick();

            for (source_index, destination_index) in self.graph.edges.iter() {
                if source_index.node_index() != *node_index {
                    continue;
                }

                let source = self.graph.node(&source_index.node_index());
                let output = source.read(source_index.producer());
                let destination = self
                    .graph
                    .nodes
                    .get_mut(&destination_index.node_index())
                    .unwrap();
                destination.write(destination_index.consumer(), output);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::{self, FeedbackSinkProducer, FeedbackSourceConsumer};
    use crate::graph::{ConsumerIndex, ProducerIndex};
    use crate::node::{ExternalConsumer, ExternalNodeWrapper, ExternalProducer, Node, NodeWrapper};

    type Payload = i32;

    struct Number(Payload);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum NumberInput {}

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct NumberOutput;

    impl Node<Payload> for Number {
        type Consumer = NumberInput;
        type Producer = NumberOutput;

        fn read(&self, _producer: Self::Producer) -> Payload {
            self.0
        }
    }

    impl From<Number> for SignalNode<TestNode> {
        fn from(number: Number) -> Self {
            Self::RegisteredNode(TestNode::Number(number))
        }
    }

    impl From<NumberInput> for SignalNodeConsumer<TestConsumer> {
        fn from(number: NumberInput) -> Self {
            Self::RegisteredNode(TestConsumer::Number(number))
        }
    }

    impl From<NumberOutput> for SignalNodeProducer<TestProducer> {
        fn from(number: NumberOutput) -> Self {
            Self::RegisteredNode(TestProducer::Number(number))
        }
    }

    #[derive(Default)]
    struct Plus {
        input1: Payload,
        input2: Payload,
        output: Payload,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum PlusInput {
        In1,
        In2,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct PlusOutput;

    impl Node<Payload> for Plus {
        type Consumer = PlusInput;
        type Producer = PlusOutput;

        fn tick(&mut self) {
            self.output = self.input1 + self.input2;
        }

        fn read(&self, _producer: Self::Producer) -> Payload {
            self.output
        }

        fn write(&mut self, consumer: Self::Consumer, input: Payload) {
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

    impl From<PlusInput> for SignalNodeConsumer<TestConsumer> {
        fn from(plus: PlusInput) -> Self {
            Self::RegisteredNode(TestConsumer::Plus(plus))
        }
    }

    impl From<PlusOutput> for SignalNodeProducer<TestProducer> {
        fn from(plus: PlusOutput) -> Self {
            Self::RegisteredNode(TestProducer::Plus(plus))
        }
    }

    #[derive(Default)]
    struct Recorder(Payload);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderInput;

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderOutput;

    impl Node<Payload> for Recorder {
        type Consumer = RecorderInput;
        type Producer = RecorderOutput;

        fn read(&self, _producer: Self::Producer) -> Payload {
            self.0
        }

        fn write(&mut self, _consumer: Self::Consumer, input: Payload) {
            self.0 = input;
        }
    }

    impl From<Recorder> for SignalNode<TestNode> {
        fn from(recorder: Recorder) -> Self {
            Self::RegisteredNode(TestNode::Recorder(recorder))
        }
    }

    impl From<RecorderInput> for SignalNodeConsumer<TestConsumer> {
        fn from(recorder: RecorderInput) -> Self {
            Self::RegisteredNode(TestConsumer::Recorder(recorder))
        }
    }

    impl From<RecorderOutput> for SignalNodeProducer<TestProducer> {
        fn from(recorder: RecorderOutput) -> Self {
            Self::RegisteredNode(TestProducer::Recorder(recorder.into()))
        }
    }

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

    impl NodeWrapper for TestNode {
        type Payload = Payload;
        type Consumer = TestConsumer;
        type Producer = TestProducer;

        fn tick(&mut self) {
            match self {
                Self::Number(number) => number.tick(),
                Self::Plus(plus) => plus.tick(),
                Self::Recorder(recorder) => recorder.tick(),
            }
        }

        fn read<IntoP>(&self, producer: IntoP) -> Payload
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

        fn write<IntoC>(&mut self, consumer: IntoC, input: Payload)
        where
            IntoC: Into<Self::Consumer>,
        {
            let consumer = consumer.into();
            match self {
                Self::Number(_) => panic!("Bad bad, not good"),
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

    impl ExternalNodeWrapper<Payload> for TestNode {}

    // TODO: Can we move this and its implementation to a lib too?
    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct TestNodeIndex {
        index: usize,
        // TODO: Keep the Node class too, so we can verify that the consumer belongs to it
    }

    impl NodeIndex for TestNodeIndex {
        type Class = TestNodeClass;
        type Consumer = TestConsumer;
        type ConsumerIndex = TestConsumerIndex;
        type Producer = TestProducer;
        type ProducerIndex = TestProducerIndex;

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

    type TestSignalGraph = SignalGraph<
        SignalNode<TestNode>,
        SignalNodeIndex<TestNodeIndex>,
        SignalNodeConsumerIndex<TestConsumerIndex>,
        SignalNodeProducerIndex<TestProducerIndex>,
    >;

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

        source.write(FeedbackSourceConsumer, 10);
        source.tick();
        sink.tick();
        assert_eq!(sink.read(FeedbackSinkProducer), 10);
    }

    #[test]
    fn write_tick_read_registered_signal_node() {
        let mut node: SignalNode<TestNode> = Plus::default().into();

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
