#[macro_export]
macro_rules! graphity {
    ( $graph:ident <$payload:ty>; $( $node:ident ),* $(,)? ) => {
        // TODO: Use all without imports
        use graphity::feedback::{self, FeedbackSinkProducer, FeedbackSourceConsumer};
        use graphity::graph::{CommonConsumerIndex, CommonProducerIndex, NodeIndex, ProducerIndex, ConsumerIndex};
        use graphity::node::{
            ExternalConsumer, ExternalNodeWrapper, ExternalProducer, Node, NodeWrapper, NodeClass,
        };
        use graphity::signal::{SignalNode, SignalNodeConsumer, SignalNodeProducer, SignalGraph, SignalNodeConsumerIndex, SignalNodeProducerIndex, SignalNodeIndex};

        $(
        impl From<$node> for GeneratedNode {
            fn from(node: $node) -> Self {
                GeneratedNode::$node(node)
            }
        }

        impl From<<$node as Node<$payload>>::Consumer> for GeneratedConsumer {
            fn from(consumer: <$node as Node<$payload>>::Consumer) -> Self {
                GeneratedConsumer::$node(consumer)
            }
        }

        impl From<<$node as Node<$payload>>::Producer> for GeneratedProducer {
            fn from(producer: <$node as Node<$payload>>::Producer) -> Self {
                GeneratedProducer::$node(producer)
            }
        }
        )*

        pub enum GeneratedNode {
            $(
            $node($node),
            )*
        }

        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        pub enum GeneratedClass {
            $(
            $node,
            )*
        }

        impl NodeClass for GeneratedNode {
            type Class = GeneratedClass;

            fn class(&self) -> Self::Class {
                match self {
                    $(
                    Self::$node(_) => GeneratedClass::$node,
                    )*
                }
            }
        }

        impl NodeWrapper for GeneratedNode {
            type Payload = $payload;
            type Consumer = GeneratedConsumer;
            type Producer = GeneratedProducer;

            fn tick(&mut self) {
                match self {
                    $(
                    Self::$node(node) => node.tick(),
                    )*
                }
            }

            fn read<IntoP>(&self, producer: IntoP) -> $payload
            where
                IntoP: Into<Self::Producer>,
            {
                let producer = producer.into();
                match self {
                    $(
                    Self::$node(node) => match producer {
                        Self::Producer::$node(producer) => node.read(producer),
                        #[allow(unreachable_patterns)]
                        _ => unreachable!("Node does not offer such producer"),
                    },
                    )*
                }
            }

            fn write<IntoC>(&mut self, consumer: IntoC, input: $payload)
            where
                IntoC: Into<Self::Consumer>,
            {
                let consumer = consumer.into();
                match self {
                    $(
                    Self::$node(node) => match consumer {
                        Self::Consumer::$node(consumer) => node.write(consumer, input),
                        #[allow(unreachable_patterns)]
                        _ => unreachable!("Node does not offer such consumer"),
                    },
                    )*
                }
            }
        }

        impl ExternalNodeWrapper<$payload> for GeneratedNode {}

        #[derive(PartialEq, Eq, Copy, Clone, Hash)]
        pub struct GeneratedNodeIndex {
            class: GeneratedClass,
            index: usize,
        }

        impl NodeIndex for GeneratedNodeIndex {
            type Class = GeneratedClass;
            type Consumer = GeneratedConsumer;
            type ConsumerIndex = GeneratedConsumerIndex;
            type Producer = GeneratedProducer;
            type ProducerIndex = GeneratedProducerIndex;

            fn new(class: GeneratedClass, index: usize) -> Self {
                Self { class, index }
            }

            // TODO : Check class
            fn consumer<IntoC>(&self, consumer: IntoC) -> GeneratedConsumerIndex
            where
                IntoC: Into<GeneratedConsumer>,
            {
                let consumer = consumer.into();
                match self.class {
                    $(
                    Self::Class::$node => match consumer {
                        Self::Consumer::$node(_) => CommonConsumerIndex::new(*self, consumer),
                        #[allow(unreachable_patterns)]
                        _ => panic!("Node does not offer such consumer")
                    },
                    )*
                }
            }

            fn producer<IntoP>(&self, producer: IntoP) -> GeneratedProducerIndex
            where
                IntoP: Into<GeneratedProducer>,
            {
                let producer = producer.into();
                match self.class {
                    $(
                    Self::Class::$node => match producer {
                        Self::Producer::$node(_) => CommonProducerIndex::new(*self, producer),
                        #[allow(unreachable_patterns)]
                        _ => panic!("Node does not offer such producer")
                    },
                    )*
                }
            }
        }

        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        pub enum GeneratedConsumer {
            $(
            $node(<$node as Node<$payload>>::Consumer),
            )*
        }

        impl ExternalConsumer for GeneratedConsumer {}

        pub type GeneratedConsumerIndex = CommonConsumerIndex<GeneratedNodeIndex>;

        #[derive(PartialEq, Eq, Copy, Clone, Hash)]
        pub enum GeneratedProducer {
            $(
            $node(<$node as Node<$payload>>::Producer),
            )*
        }

        impl ExternalProducer for GeneratedProducer {}

        pub type GeneratedProducerIndex = CommonProducerIndex<GeneratedNodeIndex>;

        pub type $graph = SignalGraph<
            GeneratedNode,
            GeneratedNodeIndex,
            GeneratedConsumerIndex,
            GeneratedProducerIndex,
        >;
    };
}

// #[cfg(test)]
// mod tests {
//     // We cannot user super::* due to `graphity::...` calls in the macro.
//     use graphity::*;

//     pub struct Number(i32);

//     #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//     pub struct NumberOutput;

//     impl Node<i32> for Number {
//         type Consumer = NoConsumer;
//         type Producer = NumberOutput;

//         fn read(&self, _producer: Self::Producer) -> i32 {
//             self.0
//         }
//     }

//     #[derive(Default)]
//     pub struct Plus {
//         input1: i32,
//         input2: i32,
//         output: i32,
//     }

//     #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//     pub enum PlusInput {
//         In1,
//         In2,
//     }

//     #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//     pub struct PlusOutput;

//     impl Node<i32> for Plus {
//         type Consumer = PlusInput;
//         type Producer = PlusOutput;

//         fn tick(&mut self) {
//             self.output = self.input1 + self.input2;
//         }

//         fn write(&mut self, consumer: Self::Consumer, input: i32) {
//             match consumer {
//                 PlusInput::In1 => self.input1 = input,
//                 PlusInput::In2 => self.input2 = input,
//             }
//         }

//         fn read(&self, _producer: Self::Producer) -> i32 {
//             self.output
//         }
//     }

//     #[derive(Default)]
//     pub struct Recorder(i32);

//     #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//     pub struct RecorderInput;

//     #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
//     pub struct RecorderOutput;

//     impl Node<i32> for Recorder {
//         type Consumer = RecorderInput;
//         type Producer = RecorderOutput;

//         fn read(&self, _producer: Self::Producer) -> i32 {
//             self.0
//         }

//         fn write(&mut self, _consumer: Self::Consumer, input: i32) {
//             self.0 = input;
//         }
//     }

//     mod node {
//         use super::*;

//         #[test]
//         fn positive_input_output_flow() {
//             let mut node = Plus::default();

//             node.write(PlusInput::In1, 1);
//             node.write(PlusInput::In2, 2);
//             node.tick();

//             assert_eq!(node.read(PlusOutput), 3);
//         }
//     }

//     mod graph {
//         use super::*;

//         // Simple tree:
//         //
//         //    [Rec]
//         //      |
//         //     [+]
//         //    /   \
//         //  [1]   [2]
//         //
//         // Should save 3 to the end consumer.
//         #[test]
//         fn simple_tree() {
//             mod g {
//                 use super::{Number, Plus, Recorder};
//                 graphity!(Graph<i32>; Number, Plus, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));
//             let two = graph.add_node(Number(2));
//             let plus = graph.add_node(Plus::default());
//             let recorder = graph.add_node(Recorder::default());
//             graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
//             graph.add_edge(two.producer(NumberOutput), plus.consumer(PlusInput::In2));
//             graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 0);

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 3);
//         }

//         // Graph with 2 end consumers:
//         //
//         //  [Rec]   [Rec]
//         //      \   /
//         //       [+]
//         //      /   \
//         //    [1]   [2]
//         //
//         // Should save 3 to both.
//         #[test]
//         fn multiple_consumers() {
//             mod g {
//                 use super::{Number, Plus, Recorder};
//                 graphity!(Graph<i32>; Number, Plus, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));
//             let two = graph.add_node(Number(2));
//             let plus = graph.add_node(Plus::default());
//             let recorder1 = graph.add_node(Recorder::default());
//             let recorder2 = graph.add_node(Recorder::default());
//             graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
//             graph.add_edge(two.producer(NumberOutput), plus.consumer(PlusInput::In2));
//             graph.add_edge(plus.producer(PlusOutput), recorder1.consumer(RecorderInput));
//             graph.add_edge(plus.producer(PlusOutput), recorder2.consumer(RecorderInput));

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder1).unwrap().read(RecorderOutput), 0);
//             assert_eq!(graph.get_node(&recorder1).unwrap().read(RecorderOutput), 0);

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder2).unwrap().read(RecorderOutput), 3);
//             assert_eq!(graph.get_node(&recorder2).unwrap().read(RecorderOutput), 3);
//         }

//         // Graph with a loop:
//         //
//         //  [Rec]    __
//         //      \   /  |
//         //       [+]   V
//         //      /   \__|
//         //    [1]
//         //
//         // Should feedback and keep increasing the recorded value.
//         //#[test]
//         // XXX: Skip this one for now, there is a flake due to random ordering of nodes
//         fn internal_cycle() {
//             mod g {
//                 use super::{Number, Plus, Recorder};
//                 graphity!(Graph<i32>; Number, Plus, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));
//             let plus = graph.add_node(Plus::default());
//             let recorder = graph.add_node(Recorder::default());
//             graph.add_edge(one.producer(NumberOutput), plus.consumer(PlusInput::In1));
//             graph.add_edge(plus.producer(PlusOutput), plus.consumer(PlusInput::In2));
//             graph.add_edge(plus.producer(PlusOutput), recorder.consumer(RecorderInput));

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 0);

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 1);

//             graph.tick();
//             assert_eq!(graph.get_node(&recorder).unwrap().read(RecorderOutput), 2);
//         }

//         #[test]
//         fn add_and_get_node() {
//             mod g {
//                 use super::Number;
//                 graphity!(Graph<i32>; Number);
//             }

//             let mut graph = g::Graph::new();

//             let one = graph.add_node(Number(1));
//             assert!(graph.get_node(&one).is_some());
//         }

//         #[test]
//         fn get_nonexistent_node() {
//             mod g {
//                 use super::Number;
//                 graphity!(Graph<i32>; Number);
//             }

//             let one = {
//                 let mut graph = g::Graph::new();
//                 graph.add_node(Number(1))
//             };
//             let graph = g::Graph::new();

//             assert!(graph.get_node(&one).is_none());
//         }

//         #[test]
//         fn read_node() {
//             mod g {
//                 use super::Number;
//                 graphity!(Graph<i32>; Number);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));

//             assert_eq!(graph.get_node(&one).unwrap().read(NumberOutput), 1);
//         }

//         #[test]
//         #[should_panic(expected = "Node does not provide given producer")]
//         fn panic_on_read_nonexistent_producer() {
//             mod g {
//                 use super::{Number, Recorder};
//                 graphity!(Graph<i32>; Number, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));

//             assert_eq!(graph.get_node(&one).unwrap().read(RecorderOutput), 1);
//         }

//         #[test]
//         fn get_consumer_index() {
//             mod g {
//                 use super::Plus;
//                 graphity!(Graph<i32>; Plus);
//             }

//             let mut graph = g::Graph::new();
//             let plus = graph.add_node(Plus::default());

//             plus.consumer(PlusInput::In1);
//         }

//         #[test]
//         #[should_panic(expected = "Node does not provide given consumer")]
//         fn panic_on_get_invalid_consumer_index() {
//             mod g {
//                 use super::{Plus, Recorder};
//                 graphity!(Graph<i32>; Plus, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let plus = graph.add_node(Plus::default());

//             plus.consumer(RecorderInput);
//         }

//         #[test]
//         fn get_producer_index() {
//             mod g {
//                 use super::Plus;
//                 graphity!(Graph<i32>; Plus);
//             }

//             let mut graph = g::Graph::new();
//             let plus = graph.add_node(Plus::default());

//             plus.producer(PlusOutput);
//         }

//         #[test]
//         #[should_panic(expected = "Node does not provide given producer")]
//         fn panic_on_get_invalid_producer_index() {
//             mod g {
//                 use super::{Plus, Recorder};
//                 graphity!(Graph<i32>; Plus, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let plus = graph.add_node(Plus::default());

//             plus.producer(RecorderOutput);
//         }

//         #[test]
//         fn add_edge() {
//             mod g {
//                 use super::{Number, Recorder};
//                 graphity!(Graph<i32>; Number, Recorder);
//             }

//             let mut graph = g::Graph::new();
//             let one = graph.add_node(Number(1));
//             let recorder = graph.add_node(Recorder::default());

//             graph.add_edge(one.producer(NumberOutput), recorder.consumer(RecorderInput));
//         }
//     }
// }
