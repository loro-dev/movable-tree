//! This example is used to test the memory usage of inserting n nodes
//!
//! It takes 39MB to record the history of inserting 100K nodes
//! It takes 450MB to record the history of inserting 1M nodes
//!
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;
use std::thread::spawn;

use movable_tree::Forest;
pub fn main() {
    let profiler = dhat::Profiler::builder().trim_backtraces(None).build();
    let mut forest: Forest<usize> = Forest::new();
    let mut history = vec![];
    forest.mov(0, None).unwrap();
    for i in 0..1_000_000 {
        history.push(forest.clone());
        forest.mov(i + 1, Some(i)).unwrap();
    }
    drop(profiler);
    spawn(move || drop(history));
}
