// TODO: This will contrain only documentation and reexports
// TODO: Leave graph as is
// TODO: Add module for node wrapper
// TODO: Add module that would use graph with nodes, passing the information, avoiding loops

// TODO: Implement removal of node and of edge
// when removed, try removing all feedback nodes to see if one of them becomes redundant
// TODO: Document that this models signal flow, allows for feedback loops,
// handling them with delay
// TODO: Make reverting of feedbacks more effective by finding all the loops at once
// TODO: Remove shipped NoProducer and NoConsumer, they are getting duplicated trait implementations

pub mod feedback;
pub mod graph;
mod graphity;
mod internal;
pub mod node;
pub mod signal;
mod sort;
