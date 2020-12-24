use std::hash::Hash;

// TODO: Define producer and consumer traits here too, both internal and external
// TODO: Implement class for NodeClass and then drop NodeWrapper requirement from Graph

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

pub trait ExternalNodeWrapper<T: Default>: NodeWrapper<T> {}

pub trait Node<T: Default> {
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    fn read(&self, _producer: Self::Producer) -> T {
        T::default()
    }

    fn write(&mut self, _consumer: Self::Consumer, _input: T) {}
}

pub trait ExternalConsumer: Copy + Hash {}
pub trait ExternalProducer: Copy + Hash {}
