#![allow(dead_code)]

pub mod client;
mod message;
pub mod raft;
pub mod role;

pub type NodeId = usize;
pub type Term = usize;
pub type MessageId = usize;
