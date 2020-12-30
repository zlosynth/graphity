#[macro_use]
extern crate graphity;

use graphity::{Node, NodeIndex};

mod g {
    use super::{Echo, Generator, Sum};
    graphity!(Graph<i32>; Echo, Generator, Sum);
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

    fn tick(&mut self) {
        println!("Echo: {}", self.input);
    }

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        self.input = input;
    }
}

pub struct Generator(i32);

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
pub struct Sum {
    input1: i32,
    input2: i32,
    output: i32,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum SumConsumer {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct SumProducer;

impl Node<i32> for Sum {
    type Consumer = SumConsumer;
    type Producer = SumProducer;

    fn tick(&mut self) {
        self.output = self.input1 + self.input2;
    }

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.output
    }

    fn write(&mut self, consumer: Self::Consumer, input: i32) {
        match consumer {
            Self::Consumer::In1 => self.input1 = input,
            Self::Consumer::In2 => self.input2 = input,
        }
    }
}

fn main() {
    let mut graph = g::Graph::new();

    let one = graph.add_node(Generator(1));
    let two = graph.add_node(Generator(2));
    let sum = graph.add_node(Sum::default());
    let echo = graph.add_node(Echo::default());

    println!(
        "Wiring up a simple graph:

    [Echo]
       |
      [+]
     /   \\
   [1]   [2]
"
    );
    graph.add_edge(
        one.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In1),
    );
    graph.add_edge(
        two.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In2),
    );
    graph.add_edge(sum.producer(SumProducer), echo.consumer(EchoConsumer));

    graph.tick();

    println!(
        "
Rewiring it to form a feedback:

  [Echo]  __
     \\   /  |
      [+]   V
     /   \\__|
   [1]
"
    );
    graph.remove_edge(
        two.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In2),
    );
    graph.add_edge(sum.producer(SumProducer), sum.consumer(SumConsumer::In2));

    graph.tick();
    graph.tick();
    graph.tick();
}
