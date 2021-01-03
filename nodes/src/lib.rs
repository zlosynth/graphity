use graphity::Node;

#[derive(Default)]
pub struct Sum {
    input1: i32,
    input2: i32,
    producer: i32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum SumConsumer {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct SumProducer;

impl Node<i32> for Sum {
    type Consumer = SumConsumer;
    type Producer = SumProducer;

    fn write(&mut self, consumer: Self::Consumer, input: i32) {
        match consumer {
            Self::Consumer::In1 => self.input1 = input,
            Self::Consumer::In2 => self.input2 = input,
        }
    }

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.producer
    }

    fn tick(&mut self) {
        self.producer = self.input1 + self.input2;
    }
}

pub struct Generator(i32);

impl Generator {
    pub fn new(value: i32) -> Self {
        Self(value)
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum GeneratorConsumer {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct GeneratorProducer;

impl Node<i32> for Generator {
    type Consumer = GeneratorConsumer;
    type Producer = GeneratorProducer;

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.0
    }
}

#[derive(Default)]
pub struct Echo {
    input: i32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct EchoConsumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum EchoProducer {}

impl Node<i32> for Echo {
    type Consumer = EchoConsumer;
    type Producer = EchoProducer;

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        self.input = input;
    }

    fn tick(&mut self) {
        println!("Echo: {}", self.input);
    }
}

#[derive(Default)]
pub struct Recorder(i32);

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RecorderConsumer;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct RecorderProducer;

impl Node<i32> for Recorder {
    type Consumer = RecorderConsumer;
    type Producer = RecorderProducer;

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.0
    }

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        self.0 = input;
    }
}
