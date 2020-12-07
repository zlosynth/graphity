use std::hash::Hash;

#[macro_use]
extern crate graphity;

use graphity::{Graph, NoConsumer, NoProducer, Node, NodeIndex};

mod g {
    use super::{Number, Printer};
    graphity!(ExampleGraph<i32>; Printer, Number);
}

#[derive(Default)]
pub struct Printer {
    input: i32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PrinterInput;

impl Node<i32> for Printer {
    type Consumer = PrinterInput;
    type Producer = NoProducer;

    fn tick(&mut self) {
        println!("Printer says: {}", self.input);
    }

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        self.input = input;
    }
}

pub struct Number(i32);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct NumberOutput;

impl Node<i32> for Number {
    type Consumer = NoConsumer;
    type Producer = NumberOutput;

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.0
    }
}

fn main() {
    let mut graph = g::ExampleGraph::new();

    let ten = graph.add_node(Number(10));
    let printer = graph.add_node(Printer::default());

    graph.add_edge(ten.producer(NumberOutput), printer.consumer(PrinterInput));

    graph.tick();
    graph.tick();
}
