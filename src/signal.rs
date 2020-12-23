// TODO: Implemented over graph trait
// TODO: Would apply feedback nodes if needed
// TODO: Would know how to traverse the graph
// TODO: Implement feedback
//
// TODO: Split to multiple modules for internal node, the node vessel, the graph itself, feedback

use std::collections::HashSet;
use std::hash::Hash;

use crate::feedback::*;
use crate::graph::{ConsumerIndex, Graph, NodeIndex, ProducerIndex};
use crate::node::{Node, NodeWrapper};
use crate::sort;

// TODO: Define a wrapper that would allow both NodeWrapper implementation and Feedback

pub enum NodeVessel<N>
where
    N: NodeWrapper<i32>,
{
    RegisteredNode(N),
    InternalNode(InternalNode),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeVesselInput<C>
where
    C: Copy + Hash,
{
    RegisteredNode(C),
    InternalNode(InternalNodeInput),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeVesselOutput<P>
where
    P: Copy + Hash,
{
    RegisteredNode(P),
    InternalNode(InternalNodeOutput),
}

impl<N> Node<i32> for NodeVessel<N>
where
    N: NodeWrapper<i32>,
{
    type Consumer = NodeVesselInput<N::Consumer>;
    type Producer = NodeVesselOutput<N::Producer>;

    fn tick(&mut self) {
        match self {
            Self::RegisteredNode(registered_node) => registered_node.tick(),
            Self::InternalNode(internal_node) => internal_node.tick(),
        }
    }

    fn read(&self, producer: Self::Producer) -> i32 {
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

    fn write(&mut self, consumer: Self::Consumer, input: i32) {
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

// TODO XXX Implement Node over NodeVessel

pub enum InternalNode {
    FeedbackSource(FeedbackSource),
    FeedbackSink(FeedbackSink),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalNodeInput {
    FeedbackSource(FeedbackSourceInput),
    FeedbackSink(FeedbackSinkInput),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalNodeOutput {
    FeedbackSource(FeedbackSourceOutput),
    FeedbackSink(FeedbackSinkOutput),
}

impl From<FeedbackSource> for InternalNode {
    fn from(feedback_source: FeedbackSource) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSink> for InternalNode {
    fn from(feedback_sink: FeedbackSink) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl From<FeedbackSourceInput> for InternalNodeInput {
    fn from(feedback_source: FeedbackSourceInput) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSinkInput> for InternalNodeInput {
    fn from(feedback_sink: FeedbackSinkInput) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl From<FeedbackSourceOutput> for InternalNodeOutput {
    fn from(feedback_source: FeedbackSourceOutput) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSinkOutput> for InternalNodeOutput {
    fn from(feedback_sink: FeedbackSinkOutput) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl NodeWrapper<i32> for InternalNode {
    type Consumer = InternalNodeInput;
    type Producer = InternalNodeOutput;

    fn tick(&mut self) {
        match self {
            Self::FeedbackSource(feedback_source) => feedback_source.tick(),
            Self::FeedbackSink(feedback_sink) => feedback_sink.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> i32
    where
        IntoP: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
            Self::FeedbackSource(feedback_source) => match producer {
                Self::Producer::FeedbackSource(producer) => feedback_source.read(producer),
                _ => panic!("Bad bad, not good"),
            },
            Self::FeedbackSink(feedback_sink) => match producer {
                Self::Producer::FeedbackSink(producer) => feedback_sink.read(producer),
                _ => panic!("Bad bad, not good"),
            },
        }
    }

    fn write<IntoC>(&mut self, consumer: IntoC, input: i32)
    where
        IntoC: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
            Self::FeedbackSource(feedback_source) => match consumer {
                Self::Consumer::FeedbackSource(consumer) => {
                    feedback_source.write(consumer.into(), input)
                }
                _ => panic!("Bad bad, not good"),
            },
            Self::FeedbackSink(feedback_sink) => match consumer {
                Self::Consumer::FeedbackSink(consumer) => {
                    feedback_sink.write(consumer.into(), input)
                }
                _ => panic!("Bad bad, not good"),
            },
        }
    }
}

struct SignalGraph<N, NI, C, P>
where
    N: NodeWrapper<i32>,
{
    graph: Graph<N, NI, C, P>,
}

impl<N, NI, C, P> SignalGraph<N, NI, C, P>
where
    N: NodeWrapper<i32>, // Or just Node, can we support both? Do we want to?
    <N as NodeWrapper<i32>>::Producer: From<P>,
    <N as NodeWrapper<i32>>::Consumer: From<C>,
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

    // TODO: Can we drop the Eq requirement?
    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct Number(i32);

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum NumberInput {}

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
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

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum PlusInput {
        In1,
        In2,
    }

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
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

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct RecorderInput;

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
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

    // TODO: Can we move this and its implementation to a lib too?
    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    struct TestNodeIndex {
        index: usize,
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

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum TestConsumer {
        Number(NumberInput),
        Plus(PlusInput),
        Recorder(RecorderInput),
    }

    type TestConsumerIndex = ConsumerIndex<TestNodeIndex, TestConsumer, TestProducer>;

    #[derive(PartialEq, Eq, Copy, Clone, Hash)]
    enum TestProducer {
        Number(NumberOutput),
        Plus(PlusOutput),
        Recorder(RecorderOutput),
    }

    type TestProducerIndex = ProducerIndex<TestNodeIndex, TestConsumer, TestProducer>;

    type TestSignalGraph = SignalGraph<TestNode, TestNodeIndex, TestConsumer, TestProducer>;

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
        assert_eq!(graph.node(&recorder2).read(RecorderOutput), 3);
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
        //panic!("Not implemented");
    }
}
