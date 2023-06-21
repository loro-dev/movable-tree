use std::collections::HashMap;

use crate::mut_tree::Forest;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct ID {
    lamport: Lamport,
    client: Client,
}

#[derive(Debug, Clone)]
pub struct Op {
    id: ID,
    content: OpContent,
}

impl PartialEq for Op {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Op {}

impl Ord for Op {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.id.cmp(&other.id)
    }
}

impl PartialOrd for Op {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.id.cmp(&other.id))
    }
}

#[derive(Debug, Clone)]
pub enum OpContent {
    New { parent: Option<ID> },
    Move { target: ID, parent: Option<ID> },
    Delete(ID),
}

type OpLog = HashMap<Client, Vec<Op>>;
type Client = u64;
type Lamport = u32;

#[derive(Debug, Clone)]
struct OpTuple {
    op: Op,
    old_parent: Option<ID>,
}

#[derive(Debug, Clone)]
pub struct Crdt {
    forest: Forest<ID>,
    client: Client,
    next_lamport: Lamport,
    log: OpLog,
    /// ops sorted by ID
    sorted_ops: Vec<OpTuple>,
    /// the end of applied op in sorted ops.
    applied_end: usize,
}

impl Crdt {
    pub fn new(client: Client) -> Self {
        Crdt {
            client,
            forest: Default::default(),
            next_lamport: 0,
            log: Default::default(),
            sorted_ops: Default::default(),
            applied_end: 0,
        }
    }

    fn push_op(&mut self, op: Op) {
        self.log.entry(self.client).or_default().push(op.clone());
        self.sorted_ops.push(OpTuple {
            op,
            old_parent: None,
        });
    }

    fn new_id(&mut self) -> ID {
        let id = ID {
            lamport: self.next_lamport,
            client: self.client,
        };
        self.next_lamport += 1;
        id
    }

    pub fn new_node(&mut self, parent: Option<ID>) -> ID {
        let id = self.new_id();
        let op = Op {
            id,
            content: OpContent::New { parent },
        };
        self.push_op(op);
        self.apply_pending_ops();
        id
    }

    pub fn mov(&mut self, target: ID, parent: Option<ID>) {
        let id = self.new_id();
        let op = Op {
            id,
            content: OpContent::Move { target, parent },
        };
        self.push_op(op);
        self.apply_pending_ops();
    }

    pub fn delete(&mut self, target: ID) {
        let op = Op {
            id: self.new_id(),
            content: OpContent::Delete(target),
        };
        self.push_op(op);
        self.apply_pending_ops();
    }

    fn apply_pending_ops(&mut self) {
        for i in self.applied_end..self.sorted_ops.len() {
            let OpTuple { op, old_parent } = &mut self.sorted_ops[i];
            match op.content {
                OpContent::New { parent } => {
                    self.forest.mov(op.id, parent).unwrap_or_default();
                }
                OpContent::Move { target, parent } => {
                    *old_parent = self.forest.get(&target).and_then(|x| x.parent);
                    self.forest.mov(target, parent).unwrap_or_default();
                }
                OpContent::Delete(target) => {
                    self.forest.delete(target);
                }
            }
        }

        self.applied_end = self.sorted_ops.len();
    }

    #[must_use]
    fn revert_until(&mut self, id: &ID) -> Vec<Op> {
        let trim_start = match self.sorted_ops.binary_search_by_key(&id, |x| &x.op.id) {
            Ok(_) => unreachable!(),
            Err(i) => i,
        };
        let ans: Vec<OpTuple> = self.sorted_ops.drain(trim_start..).collect();
        for op in ans.iter().rev() {
            match op.op.content {
                OpContent::New { .. } => {}
                OpContent::Move { target, .. } => {
                    self.forest.mov(target, op.old_parent).unwrap_or_default();
                }
                OpContent::Delete(target) => {
                    self.forest.undo_delete(target);
                }
            }
        }

        self.applied_end = self.sorted_ops.len();
        ans.into_iter().map(|x| x.op).collect()
    }

    pub fn merge(&mut self, other: &Self) {
        let mut ans = Vec::new();
        for (client, ops) in other.log.iter() {
            let self_start = self.log.get(client).map(|v| v.len()).unwrap_or(0);
            if ops.len() > self_start {
                let entry = self.log.entry(*client).or_default();
                for op in &ops[self_start..] {
                    entry.push(op.clone());
                    ans.push(op.clone());
                    if op.id.lamport >= self.next_lamport {
                        self.next_lamport = op.id.lamport + 1;
                    }
                }
            }
        }
        if ans.is_empty() {
            return;
        }

        let start_id = ans.iter().min().unwrap();
        let mut popped = self.revert_until(&start_id.id);
        ans.append(&mut popped);
        ans.sort();
        for op in ans {
            self.sorted_ops.push(OpTuple {
                op,
                old_parent: None,
            })
        }
        self.apply_pending_ops();
    }

    pub fn forest(&self) -> &Forest<ID> {
        &self.forest
    }
}

pub mod fuzz {
    use super::{Client, Crdt};

    #[derive(Debug, Clone, Copy, arbitrary::Arbitrary)]
    pub enum Action {
        Mov(u8, u8, u8),
        Del(u8, u8),
        Sync(u8, u8),
    }

    pub fn fuzzing(n_actors: usize, actions: Vec<Action>) {
        let mut actors = Vec::new();
        let mut ids = Vec::new();
        for i in 0..n_actors {
            actors.push(Crdt::new(i as Client))
        }

        for _ in 0..256 {
            ids.push(actors[0].new_node(None));
        }

        for j in 1..n_actors {
            let (a, b) = arref::array_mut_ref!(&mut actors, [0, j]);
            b.merge(a);
        }

        for action in actions {
            match action {
                Action::Mov(client, a, b) => {
                    actors[client as usize % n_actors].mov(ids[a as usize], Some(ids[b as usize]));
                }
                Action::Del(client, a) => {
                    actors[client as usize % n_actors].delete(ids[a as usize]);
                }
                Action::Sync(a, b) => {
                    let a = a as usize % n_actors;
                    let b = b as usize % n_actors;
                    if a == b {
                        continue;
                    }

                    let (a, b) = arref::array_mut_ref!(&mut actors, [a, b]);
                    a.merge(b);
                }
            }
        }

        for i in 1..n_actors {
            let (a, b) = arref::array_mut_ref!(&mut actors, [i - 1, i]);
            a.merge(b);
            b.merge(a);
            assert_eq!(a.forest(), b.forest());
        }
    }

    use Action::*;
    #[test]
    fn fuzz_0() {
        fuzzing(4, vec![Mov(0, 0, 161), Mov(1, 161, 241)])
    }
    #[test]
    fn fuzz_1() {
        fuzzing(2, vec![Mov(1, 29, 29), Del(0, 0), Mov(0, 0, 0)])
    }
    #[test]
    fn fuzz_2() {
        fuzzing(2, vec![Mov(1, 0, 1), Del(0, 0), Del(0, 0)])
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() {
        let mut a = Crdt::new(1);
        let mut b = Crdt::new(2);
        let mut ids = Vec::new();
        for _ in 0..10 {
            ids.push(a.new_node(None));
        }
        b.merge(&a);

        a.delete(ids[0]);
        a.mov(ids[0], Some(ids[0]));
        b.mov(ids[1], Some(ids[1]));
        b.merge(&a);
        a.merge(&b);
        assert_eq!(a.forest(), b.forest());
    }

    #[test]
    fn test_cache_size() {
        let mut a = Crdt::new(1);
        let mut ids = Vec::new();
        for _ in 0..10 {
            ids.push(a.new_node(None));
        }

        for i in 0..1_000 {
            a.mov(ids[i % 10], ids[(i + 1) % 10].into());
        }
    }
}
