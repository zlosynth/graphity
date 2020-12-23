use crate::node::Node;

pub fn new_feedback_pair() -> (FeedbackSource, FeedbackSink) {
    let value = std::rc::Rc::new(std::cell::RefCell::new(0));
    (
        FeedbackSource {
            value: std::rc::Rc::clone(&value),
        },
        FeedbackSink { value },
    )
}

pub struct FeedbackSource {
    pub value: std::rc::Rc<std::cell::RefCell<i32>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSourceInput;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSourceOutput {}

impl Node<i32> for FeedbackSource {
    type Consumer = FeedbackSourceInput;
    type Producer = FeedbackSourceOutput;

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        *self.value.borrow_mut() = input;
    }
}

pub struct FeedbackSink {
    pub value: std::rc::Rc<std::cell::RefCell<i32>>,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FeedbackSinkInput {}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FeedbackSinkOutput;

impl Node<i32> for FeedbackSink {
    type Consumer = FeedbackSinkInput;
    type Producer = FeedbackSinkOutput;

    fn read(&self, _producer: Self::Producer) -> i32 {
        (*self.value.borrow()).clone()
    }
}
