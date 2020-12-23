// TODO: Add tests
// TODO: Make the type configurable
use std::cell::RefCell;
use std::rc::Rc;

use crate::node::*;

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
    pub value: Rc<RefCell<T>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSourceInput;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSourceOutput {}

impl<T> Node<T> for FeedbackSource<T>
where
    T: Default,
{
    type Consumer = FeedbackSourceInput;
    type Producer = FeedbackSourceOutput;

    fn write(&mut self, _consumer: Self::Consumer, input: T) {
        *self.value.borrow_mut() = input;
    }
}

pub struct FeedbackSink<T> {
    pub value: Rc<RefCell<T>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSinkInput {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSinkOutput;

impl<T> Node<T> for FeedbackSink<T>
where
    T: Default + Clone,
{
    type Consumer = FeedbackSinkInput;
    type Producer = FeedbackSinkOutput;

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
        let (mut source, sink) = new_feedback_pair();
        source.write(FeedbackSourceInput, 10);
        assert_eq!(sink.read(FeedbackSinkOutput), 10);
    }

    #[test]
    fn pass_data_to_sink_array_i32() {
        let (mut source, sink) = new_feedback_pair();
        source.write(FeedbackSourceInput, [10, 20]);
        assert_eq!(sink.read(FeedbackSinkOutput), [10, 20]);
    }
}
