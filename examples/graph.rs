#[macro_use]
extern crate graphity;

use graphity::NodeIndex;

// See definitions of nodes used here under nodes/src/lib.rs
use graphity_nodes::*;

graphity!(
    Graph<i32>;
    Generator = {Generator, GeneratorConsumer, GeneratorProducer},
    Sum = {Sum, SumConsumer, SumProducer},
    Echo = {Echo, EchoConsumer, EchoProducer},
);

fn main() {
    let mut graph = Graph::new();

    let one = graph.add_node(Generator::new(1));
    let two = graph.add_node(Generator::new(2));
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
    graph.must_add_edge(
        one.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In1),
    );
    graph.must_add_edge(
        two.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In2),
    );
    graph.must_add_edge(sum.producer(SumProducer), echo.consumer(EchoConsumer));

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
    graph.must_add_edge(sum.producer(SumProducer), sum.consumer(SumConsumer::In2));

    graph.tick();
    graph.tick();
    graph.tick();
}
