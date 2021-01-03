/// A macro generating custom implementation of a signal graph for given node
/// types.
///
/// # Examples
///
/// The following code illustrates how would one generate code for a graph that
/// could keep instances of `Generator` and `Echo` nodes:
///
/// ```
/// # use graphity_nodes::*;
/// # #[macro_use]
/// # extern crate graphity;
/// # fn main() {
/// // pub struct Generator ...
/// // pub struct GeneratorConsumer ...
/// // pub struct GeneratorProducer ...
/// // pub struct Echo ...
/// // pub struct EchoConsumer ...
/// // pub struct EchoProducer ...
///
/// graphity!(
///     Graph<i32>;
///     Generator = {Generator, GeneratorConsumer, GeneratorProducer},
///     Echo = {Echo, EchoConsumer, EchoProducer},
/// );
/// # }
/// ```
///
/// * `Graph` defines the name of the generated signal graph type.
/// * `<i32>` dictates the payload type that will flow between nodes.
/// * `Generator` and `Echo` on the left hand side are identificators for each
///    of the nodes. They must be unique within the graph.
/// * Triplets on their right hand side reference a node and its associated
///   consumer and producer types. Read the [Node
///   documentation](node/trait.Node.html) to learn how to define these.
///
/// Once the macro generates the signal graph type, it can be instantiated:
///
/// ```
/// # use graphity_nodes::*;
/// # #[macro_use]
/// # extern crate graphity;
/// # fn main() {
/// # graphity!(
/// #     Graph<i32>;
/// #     Generator = {Generator, GeneratorConsumer, GeneratorProducer},
/// #     Echo = {Echo, EchoConsumer, EchoProducer},
/// # );
/// let graph = Graph::new();
/// # }
/// ```
///
/// For more details on how to use such graph, see the [`SignalGraph`
/// documentation](file:///home/phoracek/code/zlosynth/graphity/target/doc/graphity/signal/struct.SignalGraph.html).
#[macro_export]
macro_rules! graphity {
    ( $graph:ident<$payload:ty>; $( $nid:ident = {$node:ty, $consumer:ty, $producer:ty} ),* $(,)? ) => {
        pub enum __Node {
            $(
            $nid($node),
            )*
        }

        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        pub enum __NodeClass {
            $(
            $nid,
            )*
        }

        impl graphity::node::NodeClass for __Node {
            type Class = __NodeClass;

            fn class(&self) -> Self::Class {
                match self {
                    $(
                    Self::$nid(_) => __NodeClass::$nid,
                    )*
                }
            }
        }

        impl graphity::node::NodeWrapper for __Node {
            type Payload = $payload;
            type Consumer = __Consumer;
            type Producer = __Producer;

            fn tick(&mut self) {
                match self {
                    $(
                    Self::$nid(node) => <$node as graphity::node::Node<$payload>>::tick(node),
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
                    Self::$nid(node) => match producer {
                        Self::Producer::$nid(producer) => <$node as graphity::node::Node<$payload>>::read(node, producer),
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
                    Self::$nid(node) => match consumer {
                        Self::Consumer::$nid(consumer) => <$node as graphity::node::Node<$payload>>::write(node, consumer, input),
                        #[allow(unreachable_patterns)]
                        _ => unreachable!("Node does not offer such consumer"),
                    },
                    )*
                }
            }
        }

        impl graphity::node::ExternalNodeWrapper<$payload> for __Node {}

        #[derive(PartialEq, Eq, Copy, Clone, Hash)]
        pub struct __NodeIndex {
            class: __NodeClass,
            index: usize,
        }

        impl graphity::node::NodeIndex for __NodeIndex {
            type Class = __NodeClass;
            type Consumer = __Consumer;
            type ConsumerIndex = __ConsumerIndex;
            type Producer = __Producer;
            type ProducerIndex = __ProducerIndex;

            fn new(class: __NodeClass, index: usize) -> Self {
                Self { class, index }
            }

            fn consumer<IntoC>(&self, consumer: IntoC) -> __ConsumerIndex
            where
                IntoC: Into<__Consumer>,
            {
                let consumer = consumer.into();
                match self.class {
                    $(
                    Self::Class::$nid => match consumer {
                        Self::Consumer::$nid(_) => <
                            graphity::node::CommonConsumerIndex<__NodeIndex> as graphity::node::ConsumerIndex
                        >::new(*self, consumer),
                        #[allow(unreachable_patterns)]
                        _ => panic!("Node does not offer such consumer")
                    },
                    )*
                }
            }

            fn producer<IntoP>(&self, producer: IntoP) -> __ProducerIndex
            where
                IntoP: Into<__Producer>,
            {
                let producer = producer.into();
                match self.class {
                    $(
                    Self::Class::$nid => match producer {
                        Self::Producer::$nid(_) => <
                            graphity::node::CommonProducerIndex<__NodeIndex> as graphity::node::ProducerIndex
                        >::new(*self, producer),
                        #[allow(unreachable_patterns)]
                        _ => panic!("Node does not offer such producer")
                    },
                    )*
                }
            }
        }

        #[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
        pub enum __Consumer {
            $(
            $nid(<$node as graphity::node::Node<$payload>>::Consumer),
            )*
        }

        impl graphity::node::ExternalConsumer for __Consumer {}

        pub type __ConsumerIndex = graphity::node::CommonConsumerIndex<__NodeIndex>;

        #[derive(PartialEq, Eq, Copy, Clone, Hash)]
        pub enum __Producer {
            $(
            $nid(<$node as graphity::node::Node<$payload>>::Producer),
            )*
        }

        impl graphity::node::ExternalProducer for __Producer {}

        pub type __ProducerIndex = graphity::node::CommonProducerIndex<__NodeIndex>;

        $(
        impl From<$node> for __Node {
            fn from(node: $node) -> Self {
                Self::$nid(node)
            }
        }

        impl From<$consumer> for __Consumer {
            fn from(consumer: <$node as graphity::node::Node<$payload>>::Consumer) -> Self {
                Self::$nid(consumer)
            }
        }

        impl From<$producer> for __Producer {
            fn from(producer: <$node as graphity::node::Node<$payload>>::Producer) -> Self {
                Self::$nid(producer)
            }
        }
        )*

        pub type $graph = graphity::signal::SignalGraph<
            __Node,
            __NodeIndex,
            __ConsumerIndex,
            __ProducerIndex,
        >;
    };
    ( $graph:ident <$payload:ty>; $( $node:ident ),* $(,)? ) => {
        compile_error!(
            "The signature of the graphity! macro has been changed. \
             Read the documentation https://docs.rs/graphity/latest/graphity/macro.graphity.html \
             to learn about the new format"
        );
    };
}

#[cfg(test)]
mod tests {
    use graphity::node::{NodeIndex, NodeWrapper};

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
        use graphity_nodes::*;

        graphity!(
            Graph<i32>;
            Generator = {Generator, GeneratorConsumer, GeneratorProducer},
            Sum = {Sum, SumConsumer, SumProducer},
            Recorder = {Recorder, RecorderConsumer, RecorderProducer},
        );

        let mut graph = Graph::new();

        let one = graph.add_node(Generator::new(1));
        let two = graph.add_node(Generator::new(2));
        let sum = graph.add_node(Sum::default());
        let recorder = graph.add_node(Recorder::default());

        graph.must_add_edge(
            one.producer(GeneratorProducer),
            sum.consumer(SumConsumer::In1),
        );
        graph.must_add_edge(
            two.producer(GeneratorProducer),
            sum.consumer(SumConsumer::In2),
        );
        graph.must_add_edge(
            sum.producer(SumProducer),
            recorder.consumer(RecorderConsumer),
        );

        graph.tick();
        assert_eq!(graph.node(&recorder).unwrap().read(RecorderProducer), 3);
    }
}
