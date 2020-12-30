# Graphity

Model signal flow between nodes within a directed graph. This library is
intended to be used for (but not limited to) sounds applications where audio
signal will flow between individual node which will be generating and editing
it.

Documentation:

* [API reference (docs.rs)](https://docs.rs/graphity)
* [Repository (github.com)](https://github.com/zlosynth/graphity)
* [Crate (crates.io)](https://crates.io/crates/graphity)

## Usage

Add the following to your `Cargo.toml`:

``` toml
[dependencies]
graphity = "1.0"
```

Then you need to:

1. Define your nodes by implementing the `Node` trait.
2. Generate a graph type to hold these nodes using a provided macro.
3. Instantiate the graph, add nodes, connect them using edges.
4. Trigger `tick` operation which will push signals through the graph.

In this example, we will use 3 node types and wire them up as following:

```
|   [1]   [2]  - Generators are outputting their given value
|     \   /
|      [+]     - Sum adds the two inputs together
|       |
V      [3]     - Echo prints its input on the standard output
```

The following snippet illustrates how would be such a graph modeled via this
library. You can find the code in its full length under
[examples/](examples/graph.rs):

``` rust
impl Node<i32> for Echo {
    ...
}

impl Node<i32> for Generator {
   ...
}

impl Node<i32> for Sum {
   ...
}

mod g {
    use super::{Echo, Generator, Sum};
    graphity!(Graph<i32>; Echo, Generator, Sum);
}

fn main() {
    let mut graph = g::Graph::new();

    let one = graph.add_node(Generator(1));
    let two = graph.add_node(Generator(2));
    let sum = graph.add_node(Sum::default());
    let echo = graph.add_node(Echo::default());

    graph.must_add_edge(
        one.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In1),
    );
    graph.must_add_edge(
        two.producer(GeneratorProducer),
        sum.consumer(SumConsumer::In2),
    );
    graph.must_add_edge(
        sum.producer(SumProducer),
        echo.consumer(EchoConsumer)
    );

    graph.tick();
}
```

You can find a detailed example and exhaustive API explanation in the
[documentation](https://docs.rs/graphity).

If you prefer tinkering with code over reading documentation, see and try
included [examples](examples/):

``` shell
cargo run --example graph
```

# License

Gazpatcho is distributed under the terms of the General Public License
version 3. See [LICENSE](LICENSE) for details.

# Changelog

Read the [CHANGELOG.md](CHANGELOG.md) to learn about changes introduced in each
release.
