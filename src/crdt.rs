use std::collections::BinaryHeap;

use im::HashMap;

use crate::{log_spaced_snapshots::LogSpacedSnapshots, Forest};

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

pub struct Crdt {
    forest: Forest<ID>,
    cache: LogSpacedSnapshots<ID, Forest<ID>>,
    client: Client,
    greatest_lamport: Lamport,
    log: OpLog,
    /// ops sorted by ID
    sorted_ops: Vec<Op>,
    /// the end of applied op in sorted ops.
    applied_end: usize,
}

impl Crdt {
    pub fn new(client: Client) -> Self {
        Crdt {
            client,
            forest: Default::default(),
            cache: Default::default(),
            greatest_lamport: 0,
            log: Default::default(),
            sorted_ops: Default::default(),
            applied_end: 0,
        }
    }

    fn push_op(&mut self, op: Op) {
        self.log.entry(self.client).or_default().push(op.clone());
        self.sorted_ops.push(op);
    }

    fn new_id(&mut self) -> ID {
        let id = ID {
            lamport: self.greatest_lamport,
            client: self.client,
        };
        self.greatest_lamport += 1;
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
            let op = &self.sorted_ops[i];
            match op.content {
                OpContent::New { parent } => {
                    self.forest.mov(op.id, parent).unwrap_or_default();
                    self.cache.push(op.id, self.forest.clone());
                }
                OpContent::Move { target, parent } => {
                    self.forest.mov(target, parent).unwrap_or_default();
                    self.cache.push(op.id, self.forest.clone());
                }
                OpContent::Delete(target) => {
                    self.forest.delete(target);
                    self.cache.push(op.id, self.forest.clone());
                }
            }
        }

        self.applied_end = self.sorted_ops.len();
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
                    if op.id.lamport > self.greatest_lamport {
                        self.greatest_lamport = op.id.lamport;
                    }
                }
            }
        }
        if ans.is_empty() {
            return;
        }

        ans.sort();
        let start_id = ans[0].id;
        match self.cache.pop_till_snapshot_lte(&start_id) {
            Some((id, snapshot)) => {
                let last = self
                    .sorted_ops
                    .binary_search_by_key(&id, |x| &x.id)
                    .unwrap();
                for op in self.sorted_ops.drain(last + 1..) {
                    ans.push(op);
                }
                self.forest = snapshot.clone();
                self.applied_end = self.sorted_ops.len();
                ans.sort();
                for op in ans {
                    self.sorted_ops.push(op);
                }
            }
            None => {
                ans.append(&mut self.sorted_ops);
                ans.sort();
                self.sorted_ops = ans;
                self.applied_end = 0;
            }
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
        fuzzing(4, vec![Sync(175, 175)])
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

        a.mov(ids[0], Some(ids[2]));
        b.merge(&a);
        b.mov(ids[3], Some(ids[1]));
        a.merge(&b);
        assert_eq!(a.forest(), b.forest());
    }
}
