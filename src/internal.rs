use crate::feedback::{
    FeedbackSink, FeedbackSinkInput, FeedbackSinkOutput, FeedbackSource, FeedbackSourceInput,
    FeedbackSourceOutput,
};
use crate::node::{Node, NodeWrapper};

pub enum InternalNode<T> {
    FeedbackSource(FeedbackSource<T>),
    FeedbackSink(FeedbackSink<T>),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalNodeInput {
    FeedbackSource(FeedbackSourceInput),
    FeedbackSink(FeedbackSinkInput),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum InternalNodeOutput {
    FeedbackSource(FeedbackSourceOutput),
    FeedbackSink(FeedbackSinkOutput),
}

impl<T> From<FeedbackSource<T>> for InternalNode<T> {
    fn from(feedback_source: FeedbackSource<T>) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl<T> From<FeedbackSink<T>> for InternalNode<T> {
    fn from(feedback_sink: FeedbackSink<T>) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl From<FeedbackSourceInput> for InternalNodeInput {
    fn from(feedback_source: FeedbackSourceInput) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSinkInput> for InternalNodeInput {
    fn from(feedback_sink: FeedbackSinkInput) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl From<FeedbackSourceOutput> for InternalNodeOutput {
    fn from(feedback_source: FeedbackSourceOutput) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSinkOutput> for InternalNodeOutput {
    fn from(feedback_sink: FeedbackSinkOutput) -> Self {
        Self::FeedbackSink(feedback_sink)
    }
}

impl<T> NodeWrapper<T> for InternalNode<T>
where
    T: Default + Clone,
{
    type Consumer = InternalNodeInput;
    type Producer = InternalNodeOutput;

    fn tick(&mut self) {
        match self {
            Self::FeedbackSource(feedback_source) => feedback_source.tick(),
            Self::FeedbackSink(feedback_sink) => feedback_sink.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> T
    where
        IntoP: Into<Self::Producer>,
    {
        let producer = producer.into();
        match self {
            Self::FeedbackSource(feedback_source) => match producer {
                Self::Producer::FeedbackSource(producer) => feedback_source.read(producer),
                _ => panic!("Node does not provide given producer"),
            },
            Self::FeedbackSink(feedback_sink) => match producer {
                Self::Producer::FeedbackSink(producer) => feedback_sink.read(producer),
                _ => panic!("Node does not provide given producer"),
            },
        }
    }

    fn write<IntoC>(&mut self, consumer: IntoC, input: T)
    where
        IntoC: Into<Self::Consumer>,
    {
        let consumer = consumer.into();
        match self {
            Self::FeedbackSource(feedback_source) => match consumer {
                Self::Consumer::FeedbackSource(consumer) => {
                    feedback_source.write(consumer.into(), input)
                }
                _ => panic!("Node does not provide given consumer"),
            },
            Self::FeedbackSink(feedback_sink) => match consumer {
                Self::Consumer::FeedbackSink(consumer) => {
                    feedback_sink.write(consumer.into(), input)
                }
                _ => panic!("Node does not provide given consumer"),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feedback;

    #[test]
    fn convert_into_internal() {
        let (source, sink) = feedback::new_feedback_pair();
        let _source: InternalNode<i32> = source.into();
        let _sink: InternalNode<i32> = sink.into();
    }

    #[test]
    fn access_nested_node_i32() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: InternalNode<_> = source.into();
        let mut sink: InternalNode<_> = sink.into();

        source.write(FeedbackSourceInput, 10);
        source.tick();
        sink.tick();
        assert_eq!(sink.read(FeedbackSinkOutput), 10);
    }

    #[test]
    fn access_nested_node_array_i32() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: InternalNode<_> = source.into();
        let mut sink: InternalNode<_> = sink.into();

        source.write(FeedbackSourceInput, [10, 20]);
        source.tick();
        sink.tick();
        sink.read(FeedbackSinkOutput);
    }
}
