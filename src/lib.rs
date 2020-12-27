// TODO: INtegrate criterion https://crates.io/crates/criterion and test graph operations and ticks
// ---- adding ton of nodes
// ---- adding edges between nodes
// ---- ticking a deep graph without branching
// ---- ticking a wide graph
// ---- ticking a graph with a lot of feedbacks
// ---- test it with f16 and [f16; 64]
// TODO: Document that this models signal flow, allows for feedback loops,
// handling them with delay
// TODO: Integrate coverage
// TODO: Limit public types

#![no_std]

extern crate alloc;

mod feedback;
pub mod graph;
mod graphity;
mod internal;
pub mod node;
pub mod signal;
mod sort;
