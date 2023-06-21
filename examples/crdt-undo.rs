use movable_tree::crdt_undo::Crdt;
use rand::{rngs::StdRng, Rng};
pub fn main() {
    let mut crdt: Crdt = Crdt::new(1);
    let mut ids = Vec::new();
    let size = 10_000;
    for _ in 0..size {
        ids.push(crdt.new_node(None));
    }
    let n = 1_000_000;
    let mut rng: StdRng = rand::SeedableRng::seed_from_u64(0);
    for _ in 0..n {
        let i = rng.gen::<usize>() % size;
        let j = rng.gen::<usize>() % size;
        crdt.mov(ids[i], Some(ids[j]));
    }
}
