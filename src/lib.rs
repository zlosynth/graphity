//#![no_std]
// TODO: no std
// TODO: Show how to implement node containing its own graph and nodes
// TODO: Add prefix with __ generated stuff once it is tested (I want to hear from linter)
// TODO: Define trait for consumer and producer (Copy+Hash) and introduce a macro for it
// TODO: Make the carried type configurable
// TODO: With this, we can get too much delay caused by long chains and async for asymetric side chains.
//       Maybe we can later improve the graph.tick so it travels through the graph, recursively getting
//       data for every block before ticking.
//       - Waterfall
//         vs
//       - All at once
// On each change of the graph we will calculate the order in which edges should be addressed -
// keep them as a queue

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

    fn tick(&mut self);

    fn read<P>(&self, producer: P) -> T
    where
        P: Into<Self::Producer>;

    fn write<C>(&mut self, consumer: C, _input: T)
    where
        C: Into<Self::Consumer>;
}

pub trait ProducerIndex: Copy + Hash {}

pub trait ConsumerIndex: Copy + Hash {}

pub trait NodeIndex {
    type Producer;
    type Consumer;
    type ProducerIndex: ProducerIndex;
    type ConsumerIndex: ConsumerIndex;

    fn producer<P>(&self, producer: P) -> Self::ProducerIndex
    where
        P: Into<Self::Producer>;

    fn consumer<C>(&self, consumer: C) -> Self::ConsumerIndex
    where
        C: Into<Self::Consumer>;
}

pub trait Graph {
    type NodeIndex: NodeIndex;
    type Node: NodeWrapper<i32>;
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
macro_rules! graphity {
    ( $y:ident; $( $x:ident ),* ) => {

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

impl graphity::ProducerIndex for GeneratedProducerIndex {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct GeneratedConsumerIndex{
    node_index: GeneratedNodeIndex,
    consumer: RegisteredConsumer,
}

impl graphity::ConsumerIndex for GeneratedConsumerIndex {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredNodeClass {
$(
    $x,
)*
}

pub enum RegisteredNode {
$(
    $x($x),
)*
}

impl RegisteredNode {
    fn class(&self) -> RegisteredNodeClass {
        match self {
        $(
            RegisteredNode::$x(_) => RegisteredNodeClass::$x,
        )*
        }
    }
}

$(
impl From<$x> for RegisteredNode {
    fn from(source: $x) -> Self {
        Self::$x(source)
    }
}
)*

impl graphity::NodeWrapper<i32> for RegisteredNode {
    type Consumer = RegisteredConsumer;
    type Producer = RegisteredProducer;

    fn tick(&mut self) {
        match self {
        $(
            Self::$x(n) => <$x as graphity::Node<i32>>::tick(n),
        )*
        }
    }

    fn read<P>(&self, producer: P) -> i32
    where
        P: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
        $(
            Self::$x(n) => match producer {
                Self::Producer::$x(p) => <$x as graphity::Node<i32>>::read(n, p),
                _ => panic!("Node does not provide given producer"),
            },
        )*
        }
    }

    fn write<C>(&mut self, consumer: C, input: i32)
    where
        C: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
        $(
            Self::$x(n) => match consumer {
                Self::Consumer::$x(c) => <$x as graphity::Node<i32>>::write(n, c, input),
                _ => panic!("Node does not provide given consumer"),
            },
        )*
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredProducer {
$(
    $x(<$x as graphity::Node<i32>>::Producer),
)*
}

$(
impl From<<$x as graphity::Node<i32>>::Producer> for RegisteredProducer {
    fn from(producer: <$x as graphity::Node<i32>>::Producer) -> Self {
        Self::$x(producer)
    }
}
)*

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum RegisteredConsumer {
$(
    $x(<$x as graphity::Node<i32>>::Consumer),
)*
}

$(
impl From<<$x as graphity::Node<i32>>::Consumer> for RegisteredConsumer {
    fn from(producer: <$x as graphity::Node<i32>>::Consumer) -> Self {
        Self::$x(producer)
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
            RegisteredNodeClass::$x => match producer {
                Self::Producer::$x(_) => GeneratedProducerIndex{
                    node_index: *self,
                    producer
                },
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
            RegisteredNodeClass::$x => match consumer {
                Self::Consumer::$x(_) => GeneratedConsumerIndex{
                    node_index: *self,
                    consumer
                },
                _ => panic!("Node does not provide given consumer"),
            },
            )*
        }
    }
}

pub struct $y {
    index_counter: usize,
    nodes: std::collections::HashMap<GeneratedNodeIndex, RegisteredNode>,
    edges: std::collections::HashSet<(GeneratedProducerIndex, GeneratedConsumerIndex)>,
}

impl $y {
    pub fn new() -> Self {
        Self {
            index_counter: 0,
            nodes: std::collections::HashMap::new(),
            edges: std::collections::HashSet::new(),
        }
    }
}

impl graphity::Graph for $y {
    type NodeIndex = GeneratedNodeIndex;
    type Node = RegisteredNode;
    type ProducerIndex = GeneratedProducerIndex;
    type ConsumerIndex = GeneratedConsumerIndex;

    fn add_node<N>(&mut self, node: N) -> Self::NodeIndex
    where
        N: Into<Self::Node>,
    {
        let node = node.into();
        let index = GeneratedNodeIndex{
            id: self.index_counter,
            node_class: node.class()
        };
        self.nodes.insert(index, node);
        self.index_counter += 1;
        index
    }

    fn get_node(&self, index: &Self::NodeIndex) -> Option<&Self::Node> {
        self.nodes.get(index)
    }

    fn add_edge<P, C>(&mut self, producer: P, consumer: C)
    where
        P: Into<Self::ProducerIndex>,
        C: Into<Self::ConsumerIndex>,
    {
        self.edges.insert((producer.into(), consumer.into()));
    }

    fn tick(&mut self) {
        self.nodes.iter_mut().for_each(|(_, n)| <RegisteredNode as graphity::NodeWrapper<i32>>::tick(n));
        for edge in self.edges.iter() {
            let source = self.nodes.get(&(edge.0).node_index).unwrap();
            let output = <RegisteredNode as graphity::NodeWrapper<i32>>::read(source, (edge.0).producer);
            let destination = self.nodes.get_mut(&(edge.1).node_index).unwrap();
            <RegisteredNode as graphity::NodeWrapper<i32>>::write(destination, (edge.1).consumer, output);
        }
    }
}

    };
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
                graphity!(Graph; Number, Plus, Recorder);
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
                graphity!(Graph; Number, Plus, Recorder);
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
        #[test]
        fn internal_cycle() {
            mod g {
                use super::{Number, Plus, Recorder};
                graphity!(Graph; Number, Plus, Recorder);
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
                graphity!(Graph; Number);
            }

            let mut graph = g::Graph::new();

            let one = graph.add_node(Number(1));
            assert!(graph.get_node(&one).is_some());
        }

        #[test]
        fn get_nonexistent_node() {
            mod g {
                use super::Number;
                graphity!(Graph; Number);
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
                graphity!(Graph; Number);
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
                graphity!(Graph; Number, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));

            assert_eq!(graph.get_node(&one).unwrap().read(RecorderOutput), 1);
        }

        #[test]
        fn get_consumer_index() {
            mod g {
                use super::Plus;
                graphity!(Graph; Plus);
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
                graphity!(Graph; Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.consumer(RecorderInput);
        }

        #[test]
        fn get_producer_index() {
            mod g {
                use super::Plus;
                graphity!(Graph; Plus);
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
                graphity!(Graph; Plus, Recorder);
            }

            let mut graph = g::Graph::new();
            let plus = graph.add_node(Plus::default());

            plus.producer(RecorderOutput);
        }

        #[test]
        fn add_edge() {
            mod g {
                use super::{Number, Recorder};
                graphity!(Graph; Number, Recorder);
            }

            let mut graph = g::Graph::new();
            let one = graph.add_node(Number(1));
            let recorder = graph.add_node(Recorder::default());

            graph.add_edge(one.producer(NumberOutput), recorder.consumer(RecorderInput));
        }
    }
}
