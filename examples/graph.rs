#[macro_use]
extern crate graphity;

use graphity::{Node, NodeIndex};

mod g {
    use super::{Number, Plus, Printer};
    graphity!(Graph<i32>; Printer, Number, Plus);
}

#[derive(Default)]
pub struct Printer {
    input: i32,
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct PrinterConsumer;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum PrinterProducer {}

impl Node<i32> for Printer {
    type Consumer = PrinterConsumer;
    type Producer = PrinterProducer;

    fn tick(&mut self) {
        println!("Printer: {}", self.input);
    }

    fn write(&mut self, _consumer: Self::Consumer, input: i32) {
        self.input = input;
    }
}

pub struct Number(i32);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum NumberConsumer {}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct NumberProducer;

impl Node<i32> for Number {
    type Consumer = NumberConsumer;
    type Producer = NumberProducer;

    fn read(&self, _producer: Self::Producer) -> i32 {
        self.0
    }
}

#[derive(Default)]
pub struct Plus {
    input1: i32,
    input2: i32,
    output: i32,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub enum PlusConsumer {
    In1,
    In2,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct PlusProducer;

impl Node<i32> for Plus {
    type Consumer = PlusConsumer;
    type Producer = PlusProducer;

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

    let one = graph.add_node(Number(1));
    let two = graph.add_node(Number(2));
    let plus = graph.add_node(Plus::default());
    let printer = graph.add_node(Printer::default());

    println!(
        "Wiring up a simple graph:

      [Printer]
          |
         [+]
        /   \\
      [1]   [2]
"
    );
    graph.add_edge(
        one.producer(NumberProducer),
        plus.consumer(PlusConsumer::In1),
    );
    graph.add_edge(
        two.producer(NumberProducer),
        plus.consumer(PlusConsumer::In2),
    );
    graph.add_edge(
        plus.producer(PlusProducer),
        printer.consumer(PrinterConsumer),
    );

    graph.tick();

    println!(
        "
Rewiring it to form a feedback:

    [Printer]  __
          \\   /  |
           [+]   V
          /   \\__|
        [1]
"
    );
    graph.remove_edge(
        two.producer(NumberProducer),
        plus.consumer(PlusConsumer::In2),
    );
    graph.add_edge(
        plus.producer(PlusProducer),
        plus.consumer(PlusConsumer::In2),
    );

    graph.tick();
    graph.tick();
    graph.tick();
}
