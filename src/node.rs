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
