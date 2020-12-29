/// A macro generating custom implementation of a signal graph for given node
/// types.
///
/// # Examples
///
/// The following code illustrates how would one generate code for a graph that
/// could keep instances of `Generator` and `Echo` nodes:
///
/// ```
/// # use graphity::Node;
/// #
/// # pub struct Generator;
/// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)] pub enum GeneratorConsumer {}
/// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)] pub enum GeneratorProducer {}
/// # impl Node<i32> for Generator {
/// #     type Consumer = GeneratorConsumer;
/// #     type Producer = GeneratorProducer;
/// # }
/// #
/// # pub struct Echo;
/// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)] pub enum EchoConsumer {}
/// # #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)] pub enum EchoProducer {}
/// # impl Node<i32> for Echo {
/// #     type Consumer = EchoConsumer;
/// #     type Producer = EchoProducer;
/// # }
/// #
/// # #[macro_use]
/// # extern crate graphity;
/// # fn main() {
/// // pub struct Generator ...
/// // pub struct Echo ...
///
/// mod g {
///     use super::{Generator, Echo};
///     graphity!(Graph<i32>; Generator, Echo);
/// }
/// # }
/// ```
///
/// * `Generator` and `Echo` are types implementing the [Node
///    trait](node/trait.Node.html).
/// * `mod g` limits a scope in which will the macro operate, this is needed to
///    avoid conflicts with the rest of the code.
/// * `use super...` is needed to bring nodes into the scope of the macro.
/// * `Graph` defines the name of the generated signal graph type.
/// * `i32` dictates the payload type that will flow between nodes.
///
/// Once the macro generates the signal graph type, it can be instantiated:
///
/// ```ignore
/// let graph = g::Graph::new();
/// ```
///
/// For more details on how to use such graph, see the [`SignalGraph`
/// documentation](file:///home/phoracek/code/zlosynth/graphity/target/doc/graphity/signal/struct.SignalGraph.html).
#[macro_export]
macro_rules! graphity {
    ( $graph:ident <$payload:ty>; $( $node:ident ),* $(,)? ) => {
        use graphity::node::{
            ExternalConsumer, ExternalNodeWrapper, ExternalProducer, Node, NodeWrapper, NodeClass,
            CommonConsumerIndex, CommonProducerIndex, NodeIndex, ProducerIndex, ConsumerIndex
        };
        use graphity::signal::SignalGraph;

        pub enum GeneratedNode {
            $(
            $node($node),
            )*
        }

        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        pub enum GeneratedNodeClass {
            $(
            $node,
            )*
        }

        impl NodeClass for GeneratedNode {
            type Class = GeneratedNodeClass;

            fn class(&self) -> Self::Class {
                match self {
                    $(
                    Self::$node(_) => GeneratedNodeClass::$node,
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
            class: GeneratedNodeClass,
            index: usize,
        }

        impl NodeIndex for GeneratedNodeIndex {
            type Class = GeneratedNodeClass;
            type Consumer = GeneratedConsumer;
            type ConsumerIndex = GeneratedConsumerIndex;
            type Producer = GeneratedProducer;
            type ProducerIndex = GeneratedProducerIndex;

            fn new(class: GeneratedNodeClass, index: usize) -> Self {
                Self { class, index }
            }

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

        pub type $graph = SignalGraph<
            GeneratedNode,
            GeneratedNodeIndex,
            GeneratedConsumerIndex,
            GeneratedProducerIndex,
        >;
    };
}

#[cfg(test)]
mod tests {
    use graphity::node::{Node, NodeIndex, NodeWrapper};

    pub struct Number(i32);

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub enum NumberInput {}

    #[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
    pub struct NumberOutput;

    impl Node<i32> for Number {
        type Consumer = NumberInput;
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
        use g::Graph;

        let mut graph = Graph::new();
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
}
