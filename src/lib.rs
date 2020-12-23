// TODO: This will contrain only documentation and reexports
// TODO: Leave graph as is
// TODO: Add module for node wrapper
// TODO: Add module that would use graph with nodes, passing the information, avoiding loops

// TODO: Implement removal of node and of edge
// when removed, try removing all feedback nodes to see if one of them becomes redundant
// TODO: Document that this models signal flow, allows for feedback loops,
// handling them with delay
// TODO: Make reverting of feedbacks more effective by finding all the loops at once
// TODO: Remove shipped NoProducer and NoConsumer, they are getting duplicated trait implementations

mod graph;
mod node;
mod signal;
pub mod sort;

use core::hash::Hash;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NoConsumer {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NoProducer {}

pub trait Node<T: Default> {
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    fn read(&self, _producer: Self::Producer) -> T {
        T::default()
    }

    fn write(&mut self, _consumer: Self::Consumer, _input: T) {}
}

pub trait NodeWrapper<T: Default> {
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;
    type Class;

    fn class(&self) -> Self::Class;

    fn tick(&mut self);

    fn read<P>(&self, producer: P) -> T
    where
        P: Into<Self::Producer>;

    fn write<C>(&mut self, consumer: C, _input: T)
    where
        C: Into<Self::Consumer>;
}

// TODO: Use associated types instead?
// TODO: Return pointer to the node_index
pub trait ProducerIndex<NI, P>: Copy + Hash + Eq
where
    NI: NodeIndex,
    P: Copy + Hash,
{
    fn node_index(&self) -> &NI;
    fn producer(&self) -> P;
}

// TODO: Use associated types instead?
pub trait ConsumerIndex<NI, C>: Copy + Hash + Eq
where
    NI: NodeIndex,
    C: Copy + Hash,
{
    fn node_index(&self) -> &NI;
    fn consumer(&self) -> C;
}

pub trait NodeIndex: Copy + Hash + Eq {
    type Producer: Copy + Hash;
    type Consumer: Copy + Hash;
    type ProducerIndex: ProducerIndex<Self, Self::Producer>;
    type ConsumerIndex: ConsumerIndex<Self, Self::Consumer>;

    fn producer<P>(&self, producer: P) -> Self::ProducerIndex
    where
        P: Into<Self::Producer>;

    fn consumer<C>(&self, consumer: C) -> Self::ConsumerIndex
    where
        C: Into<Self::Consumer>;
}

pub trait Graph<T: Default> {
    type NodeIndex: NodeIndex;
    type Node: NodeWrapper<T>;
    type ProducerIndex;
    type ConsumerIndex;

    fn add_node<N>(&mut self, node: N) -> Self::NodeIndex
    where
        N: Into<Self::Node>;

    fn get_node(&self, index: &Self::NodeIndex) -> Option<&Self::Node>;

    fn add_edge<P, C>(&mut self, producer: P, consumer: C)
    where
        P: Into<Self::ProducerIndex>,
        C: Into<Self::ConsumerIndex>;

    fn tick(&mut self);
}

#[macro_export]
macro_rules! graphity_inner {
    ( $graph:ident <$payload:ty>; $( $node:ident ),* $(,)? ) => {

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GeneratedNodeIndex{
    id: usize,
    node_class: RegisteredNodeClass
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GeneratedProducerIndex{
    node_index: GeneratedNodeIndex,
    producer: RegisteredProducer,
}

impl graphity::ProducerIndex<GeneratedNodeIndex, RegisteredProducer> for GeneratedProducerIndex {
    fn node_index(&self) -> &GeneratedNodeIndex {
        &self.node_index
    }

    fn producer(&self) -> RegisteredProducer {
        self.producer
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GeneratedConsumerIndex{
    node_index: GeneratedNodeIndex,
    consumer: RegisteredConsumer,
}

impl graphity::ConsumerIndex<GeneratedNodeIndex, RegisteredConsumer> for GeneratedConsumerIndex {
    fn node_index(&self) -> &GeneratedNodeIndex {
        &self.node_index
    }

    fn consumer(&self) -> RegisteredConsumer {
        self.consumer
    }
}


#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredNodeClass {
$(
    $node,
)*
}

pub enum RegisteredNode {
$(
    $node($node),
)*
}

$(
impl From<$node> for RegisteredNode {
    fn from(source: $node) -> Self {
        Self::$node(source)
    }
}
)*

impl graphity::NodeWrapper<$payload> for RegisteredNode {
    type Consumer = RegisteredConsumer;
    type Producer = RegisteredProducer;
    type Class = RegisteredNodeClass;

    fn class(&self) -> RegisteredNodeClass {
        match self {
        $(
            RegisteredNode::$node(_) => RegisteredNodeClass::$node,
        )*
        }
    }

    fn tick(&mut self) {
        match self {
        $(
            Self::$node(n) => <$node as graphity::Node<$payload>>::tick(n),
        )*
        }
    }

    fn read<P>(&self, producer: P) -> $payload
    where
        P: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
        $(
            Self::$node(n) => match producer {
                Self::Producer::$node(p) => <$node as graphity::Node<$payload>>::read(n, p),
                #[allow(unreachable_patterns)]
                _ => panic!("Node does not provide given producer"),
            },
        )*
        }
    }

    fn write<C>(&mut self, consumer: C, input: $payload)
    where
        C: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
        $(
            Self::$node(n) => match consumer {
                Self::Consumer::$node(c) => <$node as graphity::Node<$payload>>::write(n, c, input),
                #[allow(unreachable_patterns)]
                _ => panic!("Node does not provide given consumer"),
            },
        )*
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredProducer {
$(
    $node(<$node as graphity::Node<$payload>>::Producer),
)*
}

$(
impl From<<$node as graphity::Node<$payload>>::Producer> for RegisteredProducer {
    fn from(producer: <$node as graphity::Node<$payload>>::Producer) -> Self {
        Self::$node(producer)
    }
}
)*

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredConsumer {
$(
    $node(<$node as graphity::Node<$payload>>::Consumer),
)*
}

$(
impl From<<$node as graphity::Node<$payload>>::Consumer> for RegisteredConsumer {
    fn from(producer: <$node as graphity::Node<$payload>>::Consumer) -> Self {
        Self::$node(producer)
    }
}
)*

impl graphity::NodeIndex for GeneratedNodeIndex {
    type Producer = RegisteredProducer;
    type Consumer = RegisteredConsumer;
    type ProducerIndex = GeneratedProducerIndex;
    type ConsumerIndex = GeneratedConsumerIndex;

    fn producer<P>(&self, producer: P) -> Self::ProducerIndex
    where
        P: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self.node_class {
            $(
            RegisteredNodeClass::$node => match producer {
                Self::Producer::$node(_) => GeneratedProducerIndex{
                    node_index: *self,
                    producer
                },
                #[allow(unreachable_patterns)]
                _ => panic!("Node does not provide given producer"),
            },
            )*
        }
    }

    fn consumer<C>(&self, consumer: C) -> Self::ConsumerIndex
    where
        C: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self.node_class {
            $(
            RegisteredNodeClass::$node => match consumer {
                Self::Consumer::$node(_) => GeneratedConsumerIndex{
                    node_index: *self,
                    consumer
                },
                #[allow(unreachable_patterns)]
                _ => panic!("Node does not provide given consumer"),
            },
            )*
        }
    }
}

// TODO: Keep edges indexed by the destination
pub struct __Graph<N, NI, CI, PI> {
    index_counter: usize,

    nodes: std::collections::HashMap<NI, N>,
    edges: std::collections::HashSet<(PI, CI)>,

    // XXX:
    // dynamically add/remove the feedback node, have to keep feedback nodes to maintain previous state
    // - on remove, remove orphan feedback nodes without a feeder attached
    // - remove feedback if there is no need for it anymore (the loop is gone)
    // - on add, if loop is created, add feedback to the edge
    // 1. After added, look for loops
    // 2. If there is one, add feedback
    //
    // 1. On removal, remove each orphaned feedback
    // 2. Go over the rest of feedbacks, try to replace them with basic edge and see if there are any loops
    // 3. If not, drop the feedback

    // https://en.wikipedia.org/wiki/Directed_acyclic_graph
    // Use https://en.wikipedia.org/wiki/Topological_sorting
    // topo sorting, then tick the bottom most and feel all outgoing edges, continue to the next in queue
    // TODO: Keep topological sort of nodes
}

impl<N, NI, CI, PI> __Graph<N, NI, CI, PI> {
    pub fn new() -> Self {
        Self {
            index_counter: 0,
            nodes: std::collections::HashMap::new(),
            edges: std::collections::HashSet::new(),
        }
    }
}

impl graphity::Graph<$payload> for __Graph<RegisteredNode, GeneratedNodeIndex, GeneratedConsumerIndex, GeneratedProducerIndex>
        {
    type NodeIndex = GeneratedNodeIndex;
    type Node = RegisteredNode;
    type ProducerIndex = GeneratedProducerIndex;
    type ConsumerIndex = GeneratedConsumerIndex;

    // TODO: Recalculate the graph path strategy
    fn add_node<IntoN>(&mut self, node: IntoN) -> Self::NodeIndex
    where
        IntoN: Into<Self::Node>,
    {
        let node = node.into();
        let index = GeneratedNodeIndex{
            id: self.index_counter,
            node_class: <RegisteredNode as graphity::NodeWrapper<$payload>>::class(&node),
        };
        self.nodes.insert(index, node);
        self.index_counter += 1;
        index
    }

    fn get_node(&self, index: &Self::NodeIndex) -> Option<&Self::Node> {
        self.nodes.get(index)
    }

    // TODO: Reject multiple inputs to a single consumer
    fn add_edge<P, C>(&mut self, producer: P, consumer: C)
    where
        P: Into<Self::ProducerIndex>,
        C: Into<Self::ConsumerIndex>,
    {
        let edge = (producer.into(), consumer.into());

        self.edges.insert(edge.clone());

        let edges = self.edges
            .iter()
            .map(|(source, destination)| (
                <GeneratedProducerIndex as graphity::ProducerIndex<GeneratedNodeIndex, RegisteredProducer>>::node_index(source),
                <GeneratedConsumerIndex as graphity::ConsumerIndex<GeneratedNodeIndex, RegisteredConsumer>>::node_index(destination),
            ));

        if let Err(graphity::sort::Cycle) =
            graphity::sort::topological_sort(self.nodes.keys(), edges)
        {
            self.edges.remove(&edge);
            let (feedback_source, feedback_destination) = new_feedback_pair();
            let feedback_source = self.add_node(feedback_source);
            let feedback_destination = self.add_node(feedback_destination);
            self.edges.insert((edge.0, <GeneratedNodeIndex as graphity::NodeIndex>::consumer(&feedback_source, FeedbackSourceInput)));
            self.edges.insert((<GeneratedNodeIndex as graphity::NodeIndex>::producer(&feedback_destination, FeedbackDestinationOutput), edge.1));
        }
    }

    fn tick(&mut self) {
        // TODO: Use pregenerated topological sort
        // TODO: Cache this sort on every change in edges

        self.nodes.iter_mut().for_each(|(_, n)| <RegisteredNode as graphity::NodeWrapper<$payload>>::tick(n));
        for edge in self.edges.iter() {
            let source = self.nodes.get(&<GeneratedProducerIndex as graphity::ProducerIndex<GeneratedNodeIndex, RegisteredProducer>>::node_index(&edge.0)).unwrap();
            let output = <RegisteredNode as graphity::NodeWrapper<$payload>>::read(source, <GeneratedProducerIndex as graphity::ProducerIndex<GeneratedNodeIndex, RegisteredProducer>>::producer(&edge.0));
            let destination = self.nodes.get_mut(&<GeneratedConsumerIndex as graphity::ConsumerIndex<GeneratedNodeIndex, RegisteredConsumer>>::node_index(&edge.1)).unwrap();
            <RegisteredNode as graphity::NodeWrapper<$payload>>::write(destination, <GeneratedConsumerIndex as graphity::ConsumerIndex<GeneratedNodeIndex, RegisteredConsumer>>::consumer(&edge.1), output);
        }
    }
}

pub type $graph = __Graph<
    RegisteredNode,
    GeneratedNodeIndex,
    GeneratedConsumerIndex,
    GeneratedProducerIndex
>;

    };
}

// TODO: Make it only inner, not documented
// TODO: Try having this as a basic type, no macro needed

#[macro_export]
macro_rules! graphity_feedback {
    ( $payload:ty ) => {
        fn new_feedback_pair() -> (FeedbackSource, FeedbackDestination) {
            let value = std::rc::Rc::new(std::cell::RefCell::new(
                <$payload as std::default::Default>::default(),
            ));
            (
                FeedbackSource {
                    value: std::rc::Rc::clone(&value),
                },
                FeedbackDestination { value },
            )
        }

        pub struct FeedbackSource {
            pub value: std::rc::Rc<std::cell::RefCell<$payload>>,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct FeedbackSourceInput;

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum FeedbackSourceNoProducer {}

        impl graphity::Node<$payload> for FeedbackSource {
            type Consumer = FeedbackSourceInput;
            type Producer = FeedbackSourceNoProducer;

            fn write(&mut self, _consumer: Self::Consumer, input: $payload) {
                *self.value.borrow_mut() = input;
            }
        }

        pub struct FeedbackDestination {
            pub value: std::rc::Rc<std::cell::RefCell<$payload>>,
        }

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub enum FeedbackDestinationNoConsumer {}

        #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
        pub struct FeedbackDestinationOutput;

        impl graphity::Node<$payload> for FeedbackDestination {
            type Consumer = FeedbackDestinationNoConsumer;
            type Producer = FeedbackDestinationOutput;

            fn read(&self, _producer: Self::Producer) -> $payload {
                (*self.value.borrow()).clone()
            }
        }
    };
}

#[macro_export]
macro_rules! graphity {
    ( $graph:ident <$payload:ty>; $( $node:ident ),* $(,)? ) => {

graphity_feedback!($payload);

graphity_inner!(
    $graph<$payload>;
    $(
    $node,
    )*
    FeedbackSource,
    FeedbackDestination,
);

    }
}

#[cfg(test)]
mod tests {
    // We cannot user super::* due to `graphity::...` calls in the macro.
    use graphity::*;

    pub struct Number(i32);

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct NumberOutput;

    impl Node<i32> for Number {
        type Consumer = NoConsumer;
        type Producer = NumberOutput;

        fn read(&self, _producer: Self::Producer) -> i32 {
            self.0
        }
    }

    #[derive(Default)]
    pub struct Plus {
        input1: i32,
        input2: i32,
        output: i32,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum PlusInput {
        In1,
        In2,
    }

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct PlusOutput;

    impl Node<i32> for Plus {
        type Consumer = PlusInput;
        type Producer = PlusOutput;

        fn tick(&mut self) {
            self.output = self.input1 + self.input2;
        }

        fn write(&mut self, consumer: Self::Consumer, input: i32) {
            match consumer {
                PlusInput::In1 => self.input1 = input,
                PlusInput::In2 => self.input2 = input,
            }
        }

        fn read(&self, _producer: Self::Producer) -> i32 {
            self.output
        }
    }

    #[derive(Default)]
    pub struct Recorder(i32);

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct RecorderInput;

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct RecorderOutput;

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

    mod node {
        use super::*;

        #[test]
        fn positive_input_output_flow() {
            let mut node = Plus::default();

            node.write(PlusInput::In1, 1);
            node.write(PlusInput::In2, 2);
            node.tick();

            assert_eq!(node.read(PlusOutput), 3);
        }
    }

    mod graph {
        use super::*;

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
            mod g {
                use super::{Number, Plus, Recorder};
                graphity!(Graph<i32>; Number, Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));
            let two = graph.add_node(Number(2));
            let plus = graph.add_node(Plus::default());
            let recorder = graph.add_node(Recorder::default());
            graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
            graph.add_edge(two.producer(NumberOutput), plus.consumer(PlusInput::In2));
            graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

            graph.tick();
            assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 0);

            graph.tick();
            assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 3);
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
            mod g {
                use super::{Number, Plus, Recorder};
                graphity!(Graph<i32>; Number, Plus, Recorder);
            }

            let mut graph = g::Graph::new();
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
            assert_eq!(graph.get_node(&recorder1).unwrap().read(RecorderOutput), 0);
            assert_eq!(graph.get_node(&recorder1).unwrap().read(RecorderOutput), 0);

            graph.tick();
            assert_eq!(graph.get_node(&recorder2).unwrap().read(RecorderOutput), 3);
            assert_eq!(graph.get_node(&recorder2).unwrap().read(RecorderOutput), 3);
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
        //#[test]
        // XXX: Skip this one for now, there is a flake due to random ordering of nodes
        fn internal_cycle() {
            mod g {
                use super::{Number, Plus, Recorder};
                graphity!(Graph<i32>; Number, Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));
            let plus = graph.add_node(Plus::default());
            let recorder = graph.add_node(Recorder::default());
            graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
            graph.add_edge(plus.producer(PlusOutput), plus.consumer(PlusInput::In2));
            graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

            graph.tick();
            assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 0);

            graph.tick();
            assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 1);

            graph.tick();
            assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 2);
        }

        #[test]
        fn add_and_get_node() {
            mod g {
                use super::Number;
                graphity!(Graph<i32>; Number);
            }

            let mut graph = g::Graph::new();

            let one = graph.add_node(Number(1));
            assert!(graph.get_node(&one).is_some());
        }

        #[test]
        fn get_nonexistent_node() {
            mod g {
                use super::Number;
                graphity!(Graph<i32>; Number);
            }

            let one = {
                let mut graph = g::Graph::new();
                graph.add_node(Number(1))
            };
            let graph = g::Graph::new();

            assert!(graph.get_node(&one).is_none());
        }

        #[test]
        fn read_node() {
            mod g {
                use super::Number;
                graphity!(Graph<i32>; Number);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));

            assert_eq!(graph.get_node(&one).unwrap().read(NumberOutput), 1);
        }

        #[test]
        #[should_panic(expected = "Node does not provide given producer")]
        fn panic_on_read_nonexistent_producer() {
            mod g {
                use super::{Number, Recorder};
                graphity!(Graph<i32>; Number, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));

            assert_eq!(graph.get_node(&one).unwrap().read(RecorderOutput), 1);
        }

        #[test]
        fn get_consumer_index() {
            mod g {
                use super::Plus;
                graphity!(Graph<i32>; Plus);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.consumer(PlusInput::In1);
        }

        #[test]
        #[should_panic(expected = "Node does not provide given consumer")]
        fn panic_on_get_invalid_consumer_index() {
            mod g {
                use super::{Plus, Recorder};
                graphity!(Graph<i32>; Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.consumer(RecorderInput);
        }

        #[test]
        fn get_producer_index() {
            mod g {
                use super::Plus;
                graphity!(Graph<i32>; Plus);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.producer(PlusOutput);
        }

        #[test]
        #[should_panic(expected = "Node does not provide given producer")]
        fn panic_on_get_invalid_producer_index() {
            mod g {
                use super::{Plus, Recorder};
                graphity!(Graph<i32>; Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.producer(RecorderOutput);
        }

        #[test]
        fn add_edge() {
            mod g {
                use super::{Number, Recorder};
                graphity!(Graph<i32>; Number, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));
            let recorder = graph.add_node(Recorder::default());

            graph.add_edge(one.producer(NumberOutput), recorder.consumer(RecorderInput));
        }
    }
}
