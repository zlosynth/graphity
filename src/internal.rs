use crate::feedback::{FeedbackSink, FeedbackSinkProducer, FeedbackSource, FeedbackSourceConsumer};
use crate::graph::{
    CommonConsumerIndex, CommonProducerIndex, ConsumerIndex, NodeIndex, ProducerIndex,
};
use crate::node::{Node, NodeClass, NodeWrapper};

pub enum InternalNode<T> {
    FeedbackSource(FeedbackSource<T>),
    FeedbackSink(FeedbackSink<T>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalClass {
    FeedbackSource,
    FeedbackSink,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalConsumer {
    FeedbackSource(FeedbackSourceConsumer),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalProducer {
    FeedbackSink(FeedbackSinkProducer),
}

impl<T> NodeClass for InternalNode<T> {
    type Class = InternalClass;

    fn class(&self) -> Self::Class {
        match self {
            Self::FeedbackSource(_) => Self::Class::FeedbackSource,
            Self::FeedbackSink(_) => Self::Class::FeedbackSink,
        }
    }
}

impl<T> NodeWrapper for InternalNode<T>
where
    T: Default + Copy,
{
    type Payload = T;
    type Consumer = InternalConsumer;
    type Producer = InternalProducer;

    fn read<IntoP>(&self, producer: IntoP) -> T
    where
        IntoP: Into<Self::Producer>,
    {
        match self {
            Self::FeedbackSink(feedback_sink) => match producer.into() {
                Self::Producer::FeedbackSink(producer) => feedback_sink.read(producer),
            },
            Self::FeedbackSource(_) => unreachable!("Feedback source does not offer any producers"),
        }
    }

    fn write<IntoC>(&mut self, consumer: IntoC, input: T)
    where
        IntoC: Into<Self::Consumer>,
    {
        match self {
            Self::FeedbackSource(feedback_source) => match consumer.into() {
                Self::Consumer::FeedbackSource(consumer) => feedback_source.write(consumer, input),
            },
            Self::FeedbackSink(_) => unreachable!("Feedback sink does not offer any consumers"),
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct InternalNodeIndex {
    class: InternalClass,
    index: usize,
}

pub type InternalConsumerIndex = CommonConsumerIndex<InternalNodeIndex>;
pub type InternalProducerIndex = CommonProducerIndex<InternalNodeIndex>;

impl NodeIndex for InternalNodeIndex {
    type Class = InternalClass;
    type Consumer = InternalConsumer;
    type ConsumerIndex = InternalConsumerIndex;
    type Producer = InternalProducer;
    type ProducerIndex = InternalProducerIndex;

    fn new(class: Self::Class, index: usize) -> Self {
        Self { class, index }
    }

    fn consumer<IntoC>(&self, consumer: IntoC) -> InternalConsumerIndex
    where
        IntoC: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self.class {
            Self::Class::FeedbackSource => match consumer {
                Self::Consumer::FeedbackSource(_) => InternalConsumerIndex::new(*self, consumer),
            },
            Self::Class::FeedbackSink => panic!("Feedback sink does not offer any consumers"),
        }
    }

    fn producer<IntoP>(&self, producer: IntoP) -> InternalProducerIndex
    where
        IntoP: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self.class {
            Self::Class::FeedbackSink => match producer {
                Self::Producer::FeedbackSink(_) => InternalProducerIndex::new(*self, producer),
            },
            Self::Class::FeedbackSource => panic!("Feedback source does not offer any producers"),
        }
    }
}

impl<T> From<FeedbackSource<T>> for InternalNode<T> {
    fn from(feedback_source: FeedbackSource<T>) -> Self {
        InternalNode::FeedbackSource(feedback_source)
    }
}

impl<T> From<FeedbackSink<T>> for InternalNode<T> {
    fn from(feedback_sink: FeedbackSink<T>) -> Self {
        InternalNode::FeedbackSink(feedback_sink)
    }
}

impl From<FeedbackSourceConsumer> for InternalConsumer {
    fn from(feedback_source: FeedbackSourceConsumer) -> Self {
        InternalConsumer::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSinkProducer> for InternalProducer {
    fn from(feedback_sink: FeedbackSinkProducer) -> Self {
        InternalProducer::FeedbackSink(feedback_sink)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback;

    #[test]
    fn access_nested_node_i32() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: InternalNode<_> = InternalNode::FeedbackSource(source);
        let mut sink: InternalNode<_> = InternalNode::FeedbackSink(sink);

        source.write(InternalConsumer::FeedbackSource(FeedbackSourceConsumer), 10);
        source.tick();
        sink.tick();
        assert_eq!(
            sink.read(InternalProducer::FeedbackSink(FeedbackSinkProducer)),
            10
        );
    }

    #[test]
    fn access_nested_node_array_i32() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: InternalNode<_> = InternalNode::FeedbackSource(source);
        let mut sink: InternalNode<_> = InternalNode::FeedbackSink(sink);

        source.write(
            InternalConsumer::FeedbackSource(FeedbackSourceConsumer),
            [10, 20],
        );
        source.tick();
        sink.tick();
        assert_eq!(
            sink.read(InternalProducer::FeedbackSink(FeedbackSinkProducer)),
            [10, 20]
        );
    }
}
