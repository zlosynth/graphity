//! A set of traits covering registered nodes, consumers, producers and their
//! indices.

use core::hash::Hash;

#[doc(hidden)]
pub trait ExternalNodeWrapper<T: Default + Copy>: NodeWrapper<Payload = T> {}

#[doc(hidden)]
pub trait ExternalConsumer: Copy + Hash {}

#[doc(hidden)]
pub trait ExternalProducer: Copy + Hash {}

/// This trait must be implemented by the user per each node that is to be
/// registered in the signal graph.
///
/// # Example
///
/// The following code presents a `Sum` node which offers two consumers
/// `SumConsumer::In1` and `SumConsumer::In2` and one producer `SumProducer`.
/// When ticked, this node sums inputs on both consumers and writes the result
/// to the producer.
///
/// ```
/// # use graphity::Node;
/// #[derive(Default)]
/// pub struct Sum {
///     input1: i32,
///     input2: i32,
///     output: i32,
/// }
///
/// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// pub enum SumConsumer {
///     In1,
///     In2,
/// }
///
/// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
/// pub struct SumProducer;
///
/// impl Node<i32> for Sum {
///     type Consumer = SumConsumer;
///     type Producer = SumProducer;
///
///     fn tick(&mut self) {
///         self.output = self.input1 + self.input2;
///     }
///
///     fn read(&self, _producer: Self::Producer) -> i32 {
///         self.output
///     }
///
///     fn write(&mut self, consumer: Self::Consumer, input: i32) {
///         match consumer {
///             Self::Consumer::In1 => self.input1 = input,
///             Self::Consumer::In2 => self.input2 = input,
///         }
///     }
/// }
/// ```
pub trait Node<T: Default> {
    /// User-defined type allowing selection of a specific consumer (input pin)
    /// of the given node.
    ///
    /// # Example
    ///
    /// In case multiple consumers are offered by the node:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// enum ExampleConsumer {
    ///     Input1,
    ///     Input2,
    /// }
    /// ```
    ///
    /// In case only a single consumer is availble:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// struct ExampleConsumer;
    /// ```
    ///
    /// In case there are no consumers avaialble:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// enum ExampleConsumer {}
    /// ```
    type Consumer: Copy + Hash;

    /// User-defined type allowing selection of a specific producer (output pin)
    /// of the given node.
    ///
    /// # Example
    ///
    /// In case multiple producers are offered by the node:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// enum ExampleProducer {
    ///     Input1,
    ///     Input2,
    /// }
    /// ```
    ///
    /// In case only a single producer is availble:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// struct ExampleProducer;
    /// ```
    ///
    /// In case there are no producers avaialble:
    ///
    /// ```
    /// #[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
    /// enum ExampleProducer {}
    /// ```
    type Producer: Copy + Hash;

    /// Tick signalizes that all the input data were provided and the node can
    /// perform its operation on them.
    ///
    /// Default implementation does nothing, allowing users to ommit it from
    /// their implementation.
    fn tick(&mut self) {}

    /// Read data from the given producer of the node.
    ///
    /// Default implementation returns the default value of carried payload,
    /// allowing users to ommit it from their implementation.
    #[allow(unused_variables)]
    fn read(&self, producer: Self::Producer) -> T {
        T::default()
    }

    /// Write given input data into the given consumer of the node.
    ///
    /// Default implementation does nothing, allowing users to ommit it from
    /// their implementation.
    #[allow(unused_variables)]
    fn write(&mut self, consumer: Self::Consumer, input: T) {}
}

#[doc(hidden)]
pub trait NodeClass {
    type Class: Hash + Copy + Eq;

    fn class(&self) -> Self::Class;
}

/// Wrapper around the internal representation of `Node`.
///
/// This type abstracts all the nodes registered by the user. Its main
/// significance is that it is being returned from the graph when looking up
/// nodes. Read the [`Node` documentation](trait.Node.html) to learn about its
/// usage.
///
/// # Example
///
/// ```ignore
/// let node_index = graph.add_node(Generator(1));
/// let node_wrapper = graph.node(&node_index);
/// let data = node_wrapper.read(GeneratorProducer);
/// ```
pub trait NodeWrapper: NodeClass {
    type Payload: Copy + Default;
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    /// Read data from the given producer.
    ///
    /// # Panics
    ///
    /// In case the given producer does not belong to this node type, this will
    /// panic.
    #[allow(unused_variables)]
    fn read<IntoP>(&self, producer: IntoP) -> Self::Payload
    where
        IntoP: Into<Self::Producer>,
    {
        Self::Payload::default()
    }

    /// Write data into the given consumer.
    ///
    /// # Panics
    ///
    /// In case the given consumer does not belong to this node type, this will
    /// panic.
    #[allow(unused_variables)]
    fn write<IntoC>(&mut self, consumer: IntoC, _input: Self::Payload)
    where
        IntoC: Into<Self::Consumer>,
    {
    }
}

/// An index serving as a reference to a node stored in a graph.
///
/// It can be used as a reference to read or delete an existing node. It can be
/// used to access a consumer or producer index of the given node too and then
/// be used to lookup, add or remove edges in the graph.
///
/// # Example
///
/// ```ignore
/// let generator = graph.add_node(Generator(1));
/// let echo = graph.add_node(Echo::default());
///
/// graph.add_edge(
///     generator.producer(GeneratorProducer),
///     echo.consumer(EchoConsumer),
/// );
///
/// graph.remove_node(&echo);
/// ```
pub trait NodeIndex: Copy + Hash + Eq {
    #[doc(hidden)]
    type Class: Copy + Hash + Eq;
    #[doc(hidden)]
    type Consumer: Copy + Hash + Eq;
    type ConsumerIndex: ConsumerIndex<NodeIndex = Self, Consumer = Self::Consumer>;
    #[doc(hidden)]
    type Producer: Copy + Hash + Eq;
    type ProducerIndex: ProducerIndex<NodeIndex = Self, Producer = Self::Producer>;

    #[doc(hidden)]
    fn new(class: Self::Class, index: usize) -> Self;
    fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
    where
        IntoC: Into<Self::Consumer>;
    fn producer<IntoP>(&self, producer: IntoP) -> Self::ProducerIndex
    where
        IntoP: Into<Self::Producer>;
}

/// An index serving as a unique reference of a consumer of a node registered in
/// a graph.
///
/// See [`NodeIndex` documentation](trait.NodeIndex.html) to learn more.
pub trait ConsumerIndex: Copy + Hash + Eq {
    type NodeIndex: NodeIndex<Consumer = Self::Consumer>;
    type Consumer: Copy + Hash + Eq;

    #[doc(hidden)]
    fn new(node_index: Self::NodeIndex, consumer: Self::Consumer) -> Self;
    #[doc(hidden)]
    fn node_index(&self) -> Self::NodeIndex;
    #[doc(hidden)]
    fn consumer(&self) -> Self::Consumer;
}

/// An index serving as a unique reference of a producer of a node registered in
/// a graph.
///
/// See [`NodeIndex` documentation](trait.NodeIndex.html) to learn more.
pub trait ProducerIndex: Copy + Hash + Eq {
    type NodeIndex: NodeIndex<Producer = Self::Producer>;
    type Producer: Copy + Hash + Eq;

    #[doc(hidden)]
    fn new(node_index: Self::NodeIndex, producer: Self::Producer) -> Self;
    #[doc(hidden)]
    fn node_index(&self) -> Self::NodeIndex;
    #[doc(hidden)]
    fn producer(&self) -> Self::Producer;
}

#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommonConsumerIndex<NI>
where
    NI: NodeIndex,
{
    node_index: NI,
    consumer: NI::Consumer,
}

impl<NI> ConsumerIndex for CommonConsumerIndex<NI>
where
    NI: NodeIndex,
{
    type NodeIndex = NI;
    type Consumer = NI::Consumer;

    fn new(node_index: Self::NodeIndex, consumer: Self::Consumer) -> Self {
        Self {
            node_index,
            consumer,
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        self.node_index
    }

    fn consumer(&self) -> Self::Consumer {
        self.consumer
    }
}

#[doc(hidden)]
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct CommonProducerIndex<NI>
where
    NI: NodeIndex,
{
    node_index: NI,
    producer: NI::Producer,
}

impl<NI> ProducerIndex for CommonProducerIndex<NI>
where
    NI: NodeIndex,
{
    type NodeIndex = NI;
    type Producer = NI::Producer;

    fn new(node_index: Self::NodeIndex, producer: Self::Producer) -> Self {
        Self {
            node_index,
            producer,
        }
    }

    fn node_index(&self) -> Self::NodeIndex {
        self.node_index
    }

    fn producer(&self) -> Self::Producer {
        self.producer
    }
}
