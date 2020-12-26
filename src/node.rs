use std::hash::Hash;

pub trait NodeClass {
    type Class: Hash + Copy + Eq;

    fn class(&self) -> Self::Class;
}

pub trait NodeWrapper: NodeClass {
    type Payload: Copy + Default;
    type Consumer: Copy + Hash;
    type Producer: Copy + Hash;

    fn tick(&mut self) {}

    fn read<P>(&self, producer: P) -> Self::Payload
    where
        P: Into<Self::Producer>,
    {
        Self::Payload::default()
    }

    fn write<C>(&mut self, consumer: C, _input: Self::Payload)
    where
        C: Into<Self::Consumer>,
    {
    }
}

pub trait ExternalNodeWrapper<T: Default + Copy>: NodeWrapper<Payload = T> {}

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
