// TODO: This will contrain only documentation and reexports
// TODO: Rename all XClass to XNodeClass
// TODO: INtegrate criterion https://crates.io/crates/criterion and test graph operations and ticks
// ---- adding ton of nodes
// ---- adding edges between nodes
// ---- ticking a deep graph without branching
// ---- ticking a wide graph
// ---- ticking a graph with a lot of feedbacks
// ---- test it with f16 and [f16; 64]
// TODO: Document that this models signal flow, allows for feedback loops,
// handling them with delay
// TODO: Make reverting of feedbacks more effective by finding all the loops at once
// TODO: Expose public types here
// TODO: Integrate coverage
// TODO: Implement with no_std

mod feedback;
pub mod graph;
mod graphity;
mod internal;
pub mod node;
pub mod signal;
mod sort;
