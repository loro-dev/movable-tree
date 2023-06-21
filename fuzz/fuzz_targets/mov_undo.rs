#![no_main]

use libfuzzer_sys::fuzz_target;
use movable_tree::crdt_undo::fuzz::*;

fuzz_target!(|data: Vec<Action>| {
    fuzzing(4, data);
});
