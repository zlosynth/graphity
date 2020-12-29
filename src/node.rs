use core::hash::Hash;

pub trait ExternalNodeWrapper<T: Default + Copy>: NodeWrapper<Payload = T> {}
pub trait ExternalConsumer: Copy + Hash {}
pub trait ExternalProducer: Copy + Hash {}

pub trait Node<T: Default> {
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    fn read(&self, _producer: Self::Producer) -> T {
        T::default()
    }

    fn write(&mut self, _consumer: Self::Consumer, _input: T) {}
}

pub trait NodeClass {
    type Class: Hash + Copy + Eq;

    fn class(&self) -> Self::Class;
}

pub trait NodeWrapper: NodeClass {
    type Payload: Copy + Default;
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    fn read<IntoP>(&self, _producer: IntoP) -> Self::Payload
    where
        IntoP: Into<Self::Producer>,
    {
        Self::Payload::default()
    }

    fn write<IntoC>(&mut self, _consumer: IntoC, _input: Self::Payload)
    where
        IntoC: Into<Self::Consumer>,
    {
    }
}

pub trait NodeIndex: Copy + Hash + Eq {
    type Class: Copy + Hash + Eq;
    type Consumer: Copy + Hash + Eq;
    type ConsumerIndex: ConsumerIndex<NodeIndex = Self, Consumer = Self::Consumer>;
    type Producer: Copy + Hash + Eq;
    type ProducerIndex: ProducerIndex<NodeIndex = Self, Producer = Self::Producer>;

    fn new(class: Self::Class, index: usize) -> Self;
    fn consumer<IntoC>(&self, consumer: IntoC) -> Self::ConsumerIndex
    where
        IntoC: Into<Self::Consumer>;
    fn producer<IntoP>(&self, producer: IntoP) -> Self::ProducerIndex
    where
        IntoP: Into<Self::Producer>;
}

pub trait ConsumerIndex: Copy + Hash + Eq {
    type NodeIndex: NodeIndex<Consumer = Self::Consumer>;
    type Consumer: Copy + Hash + Eq;

    fn new(node_index: Self::NodeIndex, consumer: Self::Consumer) -> Self;
    fn node_index(&self) -> Self::NodeIndex;
    fn consumer(&self) -> Self::Consumer;
}

pub trait ProducerIndex: Copy + Hash + Eq {
    type NodeIndex: NodeIndex<Producer = Self::Producer>;
    type Producer: Copy + Hash + Eq;

    fn new(node_index: Self::NodeIndex, producer: Self::Producer) -> Self;
    fn node_index(&self) -> Self::NodeIndex;
    fn producer(&self) -> Self::Producer;
}

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
