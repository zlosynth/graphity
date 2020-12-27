use alloc::rc::Rc;
use core::cell::RefCell;

use crate::node::Node;

pub fn new_feedback_pair<T>() -> (FeedbackSource<T>, FeedbackSink<T>)
where
    T: Default,
{
    let value = Rc::new(RefCell::new(T::default()));
    (
        FeedbackSource {
            value: Rc::clone(&value),
        },
        FeedbackSink { value },
    )
}

pub struct FeedbackSource<T> {
    value: Rc<RefCell<T>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSourceConsumer;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSourceProducer {}

impl<T> Node<T> for FeedbackSource<T>
where
    T: Default,
{
    type Consumer = FeedbackSourceConsumer;
    type Producer = FeedbackSourceProducer;

    fn write(&mut self, _consumer: Self::Consumer, input: T) {
        *self.value.borrow_mut() = input;
    }
}

pub struct FeedbackSink<T> {
    value: Rc<RefCell<T>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSinkConsumer {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSinkProducer;

impl<T> Node<T> for FeedbackSink<T>
where
    T: Default + Clone,
{
    type Consumer = FeedbackSinkConsumer;
    type Producer = FeedbackSinkProducer;

    fn read(&self, _producer: Self::Producer) -> T {
        (*self.value.borrow()).clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize() {
        let (_source, _sink): (FeedbackSource<i32>, FeedbackSink<i32>) = new_feedback_pair();
    }

    #[test]
    fn pass_data_to_sink_i32() {
        let (mut source, mut sink) = new_feedback_pair();
        source.write(FeedbackSourceConsumer, 10);
        source.tick();
        sink.tick();
        assert_eq!(sink.read(FeedbackSinkProducer), 10);
    }

    #[test]
    fn pass_data_to_sink_array_i32() {
        let (mut source, mut sink) = new_feedback_pair();
        source.write(FeedbackSourceConsumer, [10, 20]);
        source.tick();
        sink.tick();
        assert_eq!(sink.read(FeedbackSinkProducer), [10, 20]);
    }
}
