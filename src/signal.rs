use std::collections::HashMap;
use std::convert::From;
use std::hash::Hash;

use crate::feedback::{
    self, FeedbackSink, FeedbackSinkProducer, FeedbackSource, FeedbackSourceConsumer,
};
use crate::graph::{ConsumerIndex, Graph, NodeIndex, ProducerIndex};
use crate::internal::{
    InternalClass, InternalConsumer, InternalConsumerIndex, InternalNode, InternalNodeIndex,
    InternalProducer, InternalProducerIndex,
};
use crate::node::{NodeClass, NodeWrapper};
use crate::sort;

pub enum SignalNode<N>
where
    N: NodeWrapper,
{
    Registered(N),
    Internal(InternalNode<N::Payload>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeClass<NC> {
    Registered(NC),
    Internal(InternalClass),
}

impl<N> SignalNode<N>
where
    N: NodeWrapper,
{
    fn must_registered(&self) -> &N {
        match self {
            Self::Registered(node) => node,
            _ => panic!("SignalNode is not of variant Registered"),
        }
    }

    fn must_registered_mut(&mut self) -> &mut N {
        match self {
            Self::Registered(node) => node,
            _ => panic!("SignalNode is not of variant Registered"),
        }
    }
}

impl<N> NodeClass for SignalNode<N>
where
    N: NodeClass + NodeWrapper,
{
    type Class = SignalNodeClass<N::Class>;

    fn class(&self) -> Self::Class {
        match self {
            Self::Registered(node) => Self::Class::Registered(node.class()),
            Self::Internal(node) => Self::Class::Internal(node.class()),
        }
    }
}

impl<N> NodeWrapper for SignalNode<N>
where
    N: NodeWrapper,
{
    type Payload = N::Payload;
    type Consumer = SignalConsumer<N::Consumer>;
    type Producer = SignalProducer<N::Producer>;

    fn tick(&mut self) {
        match self {
            Self::Registered(node) => node.tick(),
            Self::Internal(node) => node.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> N::Payload
    where
        IntoP: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
            Self::Registered(node) => match producer {
                Self::Producer::Registered(producer) => node.read(producer),
                _ => panic!("Node does not offer such producer"),
            },
            Self::Internal(node) => match producer {
                Self::Producer::Internal(producer) => node.read(producer),
                _ => panic!("Node does not offer such producer"),
            },
        }
    }

    fn write<IntoC>(&mut self, consumer: IntoC, input: N::Payload)
    where
        IntoC: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
            Self::Registered(node) => match consumer {
                Self::Consumer::Registered(consumer) => node.write(consumer, input),
                _ => panic!("Node does not offer such consumer"),
            },
            Self::Internal(node) => match consumer {
                Self::Consumer::Internal(consumer) => node.write(consumer, input),
                _ => panic!("Node does not offer such consumer"),
            },
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalNodeIndex<NI> {
    Registered(NI),
    Internal(InternalNodeIndex),
}

impl<NI> SignalNodeIndex<NI> {
    fn must_registered(&self) -> &NI {
        match self {
            Self::Registered(node_index) => node_index,
            _ => panic!("SignalNodeIndex is not of variant Registered"),
        }
    }
}

impl<NI> NodeIndex for SignalNodeIndex<NI>
where
    NI: NodeIndex,
{
    type Class = SignalNodeClass<NI::Class>;
    type Consumer = SignalConsumer<NI::Consumer>;
    type ConsumerIndex = SignalConsumerIndex<NI::ConsumerIndex>;
    type Producer = SignalProducer<NI::Producer>;
    type ProducerIndex = SignalProducerIndex<NI::ProducerIndex>;

    fn new(class: SignalNodeClass<NI::Class>, index: usize) -> Self {
        match class {
            Self::Class::Registered(class) => Self::Registered(NI::new(class, index)),
            Self::Class::Internal(class) => Self::Internal(InternalNodeIndex::new(class, index)),
        }
    }

    fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
    where
        IntoC: Into<Self::Consumer>,
    {
        Self::ConsumerIndex::new(*self, consumer.into())
    }

    fn producer<IntoP>(&self, producer: IntoP) -> Self::ProducerIndex
    where
        IntoP: Into<Self::Producer>,
    {
        Self::ProducerIndex::new(*self, producer.into())
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalConsumer<C>
where
    C: Copy + Hash,
{
    Registered(C),
    Internal(InternalConsumer),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalConsumerIndex<CI>
where
    CI: ConsumerIndex,
{
    Registered(CI),
    Internal(InternalConsumerIndex),
}

impl<CI> ConsumerIndex for SignalConsumerIndex<CI>
where
    CI: ConsumerIndex,
{
    type NodeIndex = SignalNodeIndex<CI::NodeIndex>;
    type Consumer = SignalConsumer<CI::Consumer>;

    fn new(node_index: Self::NodeIndex, consumer: Self::Consumer) -> Self {
        match node_index {
            Self::NodeIndex::Registered(node_index) => match consumer {
                Self::Consumer::Registered(consumer) => {
                    Self::Registered(CI::new(node_index, consumer))
                }
                _ => panic!("Node does not offer such consumer"),
            },
            Self::NodeIndex::Internal(node_index) => match consumer {
                Self::Consumer::Internal(consumer) => {
                    Self::Internal(InternalConsumerIndex::new(node_index, consumer))
                }
                _ => panic!("Node does not offer such consumer"),
            },
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        match self {
            Self::Registered(consumer_index) => {
                Self::NodeIndex::Registered(consumer_index.node_index())
            }
            Self::Internal(consumer_index) => {
                Self::NodeIndex::Internal(consumer_index.node_index())
            }
        }
    }

    fn consumer(&self) -> Self::Consumer {
        match self {
            Self::Registered(consumer_index) => {
                Self::Consumer::Registered(consumer_index.consumer())
            }
            Self::Internal(consumer_index) => Self::Consumer::Internal(consumer_index.consumer()),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SignalProducer<P>
where
    P: Copy + Hash,
{
    Registered(P),
    Internal(InternalProducer),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignalProducerIndex<CI>
where
    CI: ProducerIndex,
{
    Registered(CI),
    Internal(InternalProducerIndex),
}

impl<CI> ProducerIndex for SignalProducerIndex<CI>
where
    CI: ProducerIndex,
{
    type NodeIndex = SignalNodeIndex<CI::NodeIndex>;
    type Producer = SignalProducer<CI::Producer>;

    fn new(node_index: Self::NodeIndex, producer: Self::Producer) -> Self {
        match node_index {
            Self::NodeIndex::Registered(node_index) => match producer {
                Self::Producer::Registered(producer) => {
                    Self::Registered(CI::new(node_index, producer))
                }
                _ => panic!("Node does not offer such producer"),
            },
            Self::NodeIndex::Internal(node_index) => match producer {
                Self::Producer::Internal(producer) => {
                    Self::Internal(InternalProducerIndex::new(node_index, producer))
                }
                _ => panic!("Node does not offer such producer"),
            },
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        match self {
            Self::Registered(producer_index) => {
                Self::NodeIndex::Registered(producer_index.node_index())
            }
            Self::Internal(producer_index) => {
                Self::NodeIndex::Internal(producer_index.node_index())
            }
        }
    }

    fn producer(&self) -> Self::Producer {
        match self {
            Self::Registered(producer_index) => {
                Self::Producer::Registered(producer_index.producer())
            }
            Self::Internal(producer_index) => Self::Producer::Internal(producer_index.producer()),
        }
    }
}

#[allow(clippy::type_complexity)]
pub struct SignalGraph<N, NI, CI, PI>
where
    N: NodeWrapper<Class = NI::Class, Consumer = NI::Consumer, Producer = NI::Producer>,
    NI: NodeIndex<ConsumerIndex = CI, ProducerIndex = PI>,
    CI: ConsumerIndex<NodeIndex = NI, Consumer = NI::Consumer>,
    PI: ProducerIndex<NodeIndex = NI, Producer = NI::Producer>,
{
    graph:
        Graph<SignalNode<N>, SignalNodeIndex<NI>, SignalConsumerIndex<CI>, SignalProducerIndex<PI>>,
    feedback_edges: HashMap<
        (SignalProducerIndex<PI>, SignalConsumerIndex<CI>),
        (SignalNodeIndex<NI>, SignalNodeIndex<NI>),
    >,
    sorted_nodes: Vec<SignalNodeIndex<NI>>,
}

#[allow(clippy::new_without_default)]
impl<N, NI, CI, PI> SignalGraph<N, NI, CI, PI>
where
    N: NodeWrapper<Class = NI::Class, Consumer = NI::Consumer, Producer = NI::Producer>,
    NI: NodeIndex<ConsumerIndex = CI, ProducerIndex = PI>,
    CI: ConsumerIndex<NodeIndex = NI, Consumer = NI::Consumer>,
    PI: ProducerIndex<NodeIndex = NI, Producer = NI::Producer>,
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
        let node = SignalNode::Registered(node.into());
        let index = *self.graph.add_node(node).must_registered();
        self.update_cache();
        index
    }

    pub fn remove_node(&mut self, node_index: NI) {
        let node_index = SignalNodeIndex::Registered(node_index);
        self.graph.remove_node(node_index);
        self.update_cache();
    }

    pub fn node(&self, node_index: &NI) -> &N {
        let node_index = SignalNodeIndex::Registered(*node_index);
        self.graph.node(&node_index).must_registered()
    }

    pub fn node_mut(&mut self, node_index: &NI) -> &mut N {
        let node_index = SignalNodeIndex::Registered(*node_index);
        self.graph.node_mut(&node_index).must_registered_mut()
    }

    pub fn add_edge(&mut self, producer: PI, consumer: CI) {
        if self.has_edge(producer, consumer) {
            return;
        }

        let producer = SignalProducerIndex::Registered(producer);
        let consumer = SignalConsumerIndex::Registered(consumer);

        self.graph.add_edge(producer, consumer);

        if self.has_cycles() {
            self.graph.remove_edge(producer, consumer);
            self.add_feedback_edge(producer, consumer);
        }

        self.update_cache();
    }

    fn add_feedback_edge(
        &mut self,
        producer: SignalProducerIndex<PI>,
        consumer: SignalConsumerIndex<CI>,
    ) {
        let (source, sink) = feedback::new_feedback_pair::<N::Payload>();

        let source = self.graph.add_node(source);
        let sink = self.graph.add_node(sink);

        self.graph
            .add_edge(producer, source.consumer(FeedbackSourceConsumer));
        self.graph
            .add_edge(sink.producer(FeedbackSinkProducer), consumer);

        self.feedback_edges
            .insert((producer, consumer), (source, sink));
    }

    pub fn remove_edge(&mut self, producer: PI, consumer: CI) {
        let producer = SignalProducerIndex::Registered(producer);
        let consumer = SignalConsumerIndex::Registered(consumer);

        if self.graph.has_edge(producer, consumer) {
            self.graph.remove_edge(producer, consumer);
            self.drop_redundant_feedbacks();
        } else if self.feedback_edges.contains_key(&(producer, consumer)) {
            self.remove_feedback_edge(producer, consumer);
        }

        self.update_cache();
    }

    fn remove_feedback_edge(
        &mut self,
        producer: SignalProducerIndex<PI>,
        consumer: SignalConsumerIndex<CI>,
    ) {
        let (source, sink) = self.feedback_edges.get(&(producer, consumer)).unwrap();
        self.graph.remove_node(*source);
        self.graph.remove_node(*sink);
        self.feedback_edges.remove(&(producer, consumer));
    }

    fn drop_redundant_feedbacks(&mut self) {
        let edges = self.feedback_edges.clone();

        for ((producer, consumer), (source, sink)) in edges.iter() {
            let source_consumer = source.consumer(FeedbackSourceConsumer);
            let sink_producer = sink.producer(FeedbackSinkProducer);

            self.graph.remove_edge(*producer, source_consumer);
            self.graph.remove_edge(sink_producer, *consumer);
            self.graph.add_edge(*producer, *consumer);

            if self.has_cycles() {
                self.graph.remove_edge(*producer, *consumer);
                self.graph.add_edge(*producer, source_consumer);
                self.graph.add_edge(sink_producer, *consumer);
            } else {
                self.graph.remove_node(*source);
                self.graph.remove_node(*sink);
                self.feedback_edges.remove(&(*producer, *consumer));
            }
        }
    }

    pub fn has_edge(&mut self, producer: PI, consumer: CI) -> bool {
        let producer = SignalProducerIndex::Registered(producer);
        let consumer = SignalConsumerIndex::Registered(consumer);

        self.graph.has_edge(producer, consumer)
            || self.feedback_edges.contains_key(&(producer, consumer))
    }

    pub fn tick(&mut self) {
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

    fn update_cache(&mut self) {
        self.sorted_nodes = self.topologically_sorted_nodes().unwrap();
    }

    fn topologically_sorted_nodes(&self) -> Result<Vec<SignalNodeIndex<NI>>, sort::Cycle> {
        let nodes = self.graph.nodes.keys().copied();
        let edges = self
            .graph
            .edges
            .iter()
            .map(|(producer, consumer)| (producer.node_index(), consumer.node_index()));
        sort::topological_sort(nodes, edges)
    }

    fn has_cycles(&self) -> bool {
        self.topologically_sorted_nodes().is_err()
    }
}

impl<N> From<FeedbackSource<N::Payload>> for SignalNode<N>
where
    N: NodeWrapper,
{
    fn from(feedback_source: FeedbackSource<N::Payload>) -> Self {
        Self::Internal(InternalNode::FeedbackSource(feedback_source))
    }
}

impl<N> From<FeedbackSink<N::Payload>> for SignalNode<N>
where
    N: NodeWrapper,
{
    fn from(feedback_sink: FeedbackSink<N::Payload>) -> Self {
        Self::Internal(InternalNode::FeedbackSink(feedback_sink))
    }
}

impl<C> From<FeedbackSourceConsumer> for SignalConsumer<C>
where
    C: Hash + Copy,
{
    fn from(feedback_source: FeedbackSourceConsumer) -> Self {
        Self::Internal(InternalConsumer::FeedbackSource(feedback_source))
    }
}

impl<P> From<FeedbackSinkProducer> for SignalProducer<P>
where
    P: Hash + Copy,
{
    fn from(feedback_sink: FeedbackSinkProducer) -> Self {
        Self::Internal(InternalProducer::FeedbackSink(feedback_sink))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback::{self, FeedbackSinkProducer, FeedbackSourceConsumer};
    use crate::graph::{CommonConsumerIndex, CommonProducerIndex};
    use crate::node::{ExternalConsumer, ExternalNodeWrapper, ExternalProducer, Node, NodeWrapper};

    type Payload = i32;

    struct Number(Payload);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum NumberConsumer {}

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct NumberProducer;

    impl Node<Payload> for Number {
        type Consumer = NumberConsumer;
        type Producer = NumberProducer;

        fn read(&self, _producer: Self::Producer) -> Payload {
            self.0
        }
    }

    impl From<Number> for TestNode {
        fn from(number: Number) -> Self {
            TestNode::Number(number)
        }
    }

    impl From<NumberProducer> for TestProducer {
        fn from(number: NumberProducer) -> Self {
            TestProducer::Number(number)
        }
    }

    #[derive(Default)]
    struct Plus {
        input1: Payload,
        input2: Payload,
        output: Payload,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum PlusConsumer {
        In1,
        In2,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct PlusProducer;

    impl Node<Payload> for Plus {
        type Consumer = PlusConsumer;
        type Producer = PlusProducer;

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

    impl From<Plus> for TestNode {
        fn from(plus: Plus) -> Self {
            TestNode::Plus(plus)
        }
    }

    impl From<PlusConsumer> for TestConsumer {
        fn from(plus: PlusConsumer) -> Self {
            TestConsumer::Plus(plus)
        }
    }

    impl From<PlusProducer> for TestProducer {
        fn from(plus: PlusProducer) -> Self {
            TestProducer::Plus(plus)
        }
    }

    #[derive(Default)]
    struct Recorder(Payload);

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderConsumer;

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    struct RecorderProducer;

    impl Node<Payload> for Recorder {
        type Consumer = RecorderConsumer;
        type Producer = RecorderProducer;

        fn read(&self, _producer: Self::Producer) -> Payload {
            self.0
        }

        fn write(&mut self, _consumer: Self::Consumer, input: Payload) {
            self.0 = input;
        }
    }

    impl From<Recorder> for TestNode {
        fn from(recorder: Recorder) -> Self {
            TestNode::Recorder(recorder)
        }
    }

    impl From<RecorderConsumer> for TestConsumer {
        fn from(recorder: RecorderConsumer) -> Self {
            TestConsumer::Recorder(recorder)
        }
    }

    impl From<RecorderProducer> for TestProducer {
        fn from(recorder: RecorderProducer) -> Self {
            TestProducer::Recorder(recorder.into())
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
                    _ => panic!("Node does not offer such producer"),
                },
                Self::Plus(plus) => match producer {
                    Self::Producer::Plus(producer) => plus.read(producer),
                    _ => panic!("Node does not offer such producer"),
                },
                Self::Recorder(recorder) => match producer {
                    Self::Producer::Recorder(producer) => recorder.read(producer),
                    _ => panic!("Node does not offer such producer"),
                },
            }
        }

        fn write<IntoC>(&mut self, consumer: IntoC, input: Payload)
        where
            IntoC: Into<Self::Consumer>,
        {
            let consumer = consumer.into();
            match self {
                Self::Number(_) => panic!("Node does not offer such consumer"),
                Self::Plus(plus) => match consumer {
                    Self::Consumer::Plus(consumer) => plus.write(consumer.into(), input),
                    _ => panic!("Node does not offer such consumer"),
                },
                Self::Recorder(recorder) => match consumer {
                    Self::Consumer::Recorder(consumer) => recorder.write(consumer.into(), input),
                    _ => panic!("Node does not offer such consumer"),
                },
            }
        }
    }

    impl ExternalNodeWrapper<Payload> for TestNode {}

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct TestNodeIndex {
        class: TestNodeClass,
        index: usize,
    }

    impl NodeIndex for TestNodeIndex {
        type Class = TestNodeClass;
        type Consumer = TestConsumer;
        type ConsumerIndex = TestConsumerIndex;
        type Producer = TestProducer;
        type ProducerIndex = TestProducerIndex;

        fn new(class: TestNodeClass, index: usize) -> Self {
            Self { class, index }
        }

        fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
        where
            IntoC: Into<TestConsumer>,
        {
            let consumer = consumer.into();
            match self.class {
                Self::Class::Number => panic!("Node does not offer such consumer"),
                Self::Class::Plus => match consumer {
                    Self::Consumer::Plus(_) => Self::ConsumerIndex::new(*self, consumer.into()),
                    _ => panic!("Node does not offer such consumer"),
                },
                Self::Class::Recorder => match consumer {
                    Self::Consumer::Recorder(_) => Self::ConsumerIndex::new(*self, consumer.into()),
                    _ => panic!("Node does not offer such consumer"),
                },
            }
        }

        fn producer<IntoP>(&self, producer: IntoP) -> Self::ProducerIndex
        where
            IntoP: Into<TestProducer>,
        {
            let producer = producer.into();
            match self.class {
                Self::Class::Number => match producer {
                    Self::Producer::Number(_) => Self::ProducerIndex::new(*self, producer.into()),
                    _ => panic!("Node does not offer such producer"),
                },
                Self::Class::Plus => match producer {
                    Self::Producer::Plus(_) => Self::ProducerIndex::new(*self, producer.into()),
                    _ => panic!("Node does not offer such producer"),
                },
                Self::Class::Recorder => match producer {
                    Self::Producer::Recorder(_) => Self::ProducerIndex::new(*self, producer.into()),
                    _ => panic!("Node does not offer such producer"),
                },
            }
        }
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
    enum TestConsumer {
        Plus(PlusConsumer),
        Recorder(RecorderConsumer),
    }

    impl ExternalConsumer for TestConsumer {}

    type TestConsumerIndex = CommonConsumerIndex<TestNodeIndex>;

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum TestProducer {
        Number(NumberProducer),
        Plus(PlusProducer),
        Recorder(RecorderProducer),
    }

    impl ExternalProducer for TestProducer {}

    type TestProducerIndex = CommonProducerIndex<TestNodeIndex>;

    type TestSignalGraph =
        SignalGraph<TestNode, TestNodeIndex, TestConsumerIndex, TestProducerIndex>;

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
        let mut node: SignalNode<TestNode> = SignalNode::Registered(Plus::default().into());

        node.write(SignalConsumer::Registered(PlusConsumer::In1.into()), 10);
        node.write(SignalConsumer::Registered(PlusConsumer::In2.into()), 20);
        node.tick();
        assert_eq!(
            node.read(SignalProducer::Registered(PlusProducer.into())),
            30
        );
    }

    //
    //    [Rec]
    //      |
    //     [+]
    //    /   \
    //  [1]   [2]
    //
    #[test]
    fn tick_in_simple_tree() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let two = graph.add_node(Number(2));
        let plus = graph.add_node(Plus::default());
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            two.producer(NumberProducer),
            plus.consumer(PlusConsumer::In2),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder.consumer(RecorderConsumer),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder).read(RecorderProducer), 3);
    }

    //
    //  [Rec]   [Rec]
    //      \   /
    //       [+]
    //      /   \
    //    [1]   [2]
    //
    #[test]
    fn tick_with_multiple_consumers() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let two = graph.add_node(Number(2));
        let plus = graph.add_node(Plus::default());
        let recorder1 = graph.add_node(Recorder::default());
        let recorder2 = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            two.producer(NumberProducer),
            plus.consumer(PlusConsumer::In2),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder1.consumer(RecorderConsumer),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder2.consumer(RecorderConsumer),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder1).read(RecorderProducer), 3);
        assert_eq!(graph.node(&recorder2).read(RecorderProducer), 3);
    }

    //
    //  [Rec]    __
    //      \   /  |
    //       [+]   V
    //      /   \__|
    //    [1]
    //
    #[test]
    fn tick_with_internal_cycle() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder.consumer(RecorderConsumer),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder).read(RecorderProducer), 1);
        graph.tick();
        assert_eq!(graph.node(&recorder).read(RecorderProducer), 2);
    }

    //
    //    [Rec]
    //      |
    //     [1]
    //
    #[test]
    fn check_for_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        );

        assert!(graph.has_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        ));
    }

    //
    //    [Rec]
    //      |
    //     [1]
    //
    #[test]
    fn remove_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        );

        graph.remove_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        );

        assert!(!graph.has_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        ));
    }

    //           __
    //          /  |
    //       [+]   V
    //      /   \__|
    //    [1]
    //
    #[test]
    fn check_for_feedback_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        );

        assert!(graph.has_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        ));
        assert!(!graph.feedback_edges.is_empty());
    }

    //           __
    //          /  |
    //       [+]   V
    //      /   \__|
    //    [1]
    //
    #[test]
    fn remove_feedback_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        );

        graph.remove_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        );

        assert!(!graph.has_edge(
            plus.producer(PlusProducer),
            plus.consumer(PlusConsumer::In2),
        ));
        assert!(graph.feedback_edges.is_empty());
    }

    //           ___
    //          /   |
    //       [Rec]  |
    // del ->  |    |
    //       [Rec]  V
    //         |    |
    //        [+]   |
    //       /   \__|
    //      [1]
    //
    #[test]
    fn remove_redundant_feedback_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        let recorder1 = graph.add_node(Recorder::default());
        let recorder2 = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder1.consumer(RecorderConsumer),
        );
        graph.add_edge(
            recorder1.producer(RecorderProducer),
            recorder2.consumer(RecorderConsumer),
        );
        graph.add_edge(
            recorder2.producer(RecorderProducer),
            plus.consumer(PlusConsumer::In2),
        );

        assert!(!graph.feedback_edges.is_empty());

        graph.remove_edge(
            recorder1.producer(RecorderProducer),
            recorder2.consumer(RecorderConsumer),
        );

        assert!(graph.feedback_edges.is_empty());
    }

    //             ___
    //            /   |
    //         [Rec]  |
    //           |    |
    //         [Rec]  V
    //           |    |
    //          [+]   |
    //  del -> /   \__|
    //        [1]
    //
    #[test]
    fn do_not_remove_needed_feedback_edge() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let plus = graph.add_node(Plus::default());
        let recorder1 = graph.add_node(Recorder::default());
        let recorder2 = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        graph.add_edge(
            plus.producer(PlusProducer),
            recorder1.consumer(RecorderConsumer),
        );
        graph.add_edge(
            recorder1.producer(RecorderProducer),
            recorder2.consumer(RecorderConsumer),
        );
        graph.add_edge(
            recorder2.producer(RecorderProducer),
            plus.consumer(PlusConsumer::In2),
        );
        let original_feedbacks = graph.feedback_edges.len();

        graph.remove_edge(
            one.producer(NumberProducer),
            plus.consumer(PlusConsumer::In1),
        );
        assert_eq!(graph.feedback_edges.len(), original_feedbacks);
    }

    #[test]
    fn get_node() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));

        assert_eq!(graph.node(&one).read(NumberProducer), 1);
    }

    #[test]
    fn get_mutable_node() {
        let mut graph = TestSignalGraph::new();
        let recorder = graph.add_node(Recorder::default());

        graph.node_mut(&recorder).write(RecorderConsumer, 10);

        assert_eq!(graph.node(&recorder).read(RecorderProducer), 10);
    }

    //
    //    [Rec]
    //      |
    //     [1]
    //
    #[test]
    fn remove_node() {
        let mut graph = TestSignalGraph::new();
        let one = graph.add_node(Number(1));
        let recorder = graph.add_node(Recorder::default());
        graph.add_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        );

        graph.remove_node(recorder);

        assert!(!graph.has_edge(
            one.producer(NumberProducer),
            recorder.consumer(RecorderConsumer),
        ));
    }
}
