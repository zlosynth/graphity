use crate::feedback::{
    FeedbackSink, FeedbackSinkInput, FeedbackSinkOutput, FeedbackSource, FeedbackSourceInput,
    FeedbackSourceOutput,
};
use crate::node::{Node, NodeWrapper};

pub enum InternalNode {
    FeedbackSource(FeedbackSource<i32>),
    FeedbackSink(FeedbackSink<i32>),
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

impl From<FeedbackSource<i32>> for InternalNode {
    fn from(feedback_source: FeedbackSource<i32>) -> Self {
        Self::FeedbackSource(feedback_source)
    }
}

impl From<FeedbackSink<i32>> for InternalNode {
    fn from(feedback_sink: FeedbackSink<i32>) -> Self {
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

impl NodeWrapper<i32> for InternalNode {
    type Consumer = InternalNodeInput;
    type Producer = InternalNodeOutput;

    fn tick(&mut self) {
        match self {
            Self::FeedbackSource(feedback_source) => feedback_source.tick(),
            Self::FeedbackSink(feedback_sink) => feedback_sink.tick(),
        }
    }

    fn read<IntoP>(&self, producer: IntoP) -> i32
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

    fn write<IntoC>(&mut self, consumer: IntoC, input: i32)
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
        let _source: InternalNode = source.into();
        let _sink: InternalNode = sink.into();
    }

    #[test]
    fn access_nested_node() {
        let (source, sink) = feedback::new_feedback_pair();
        let mut source: InternalNode = source.into();
        let sink: InternalNode = sink.into();

        source.write(FeedbackSourceInput, 10);
        sink.read(FeedbackSinkOutput);
    }
}
