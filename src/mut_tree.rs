use std::{fmt::Debug, hash::Hash};

use fxhash::FxHashMap;

pub trait IdTrait: Hash + Eq + Clone + Copy + Debug {}
impl<T: Hash + Eq + Clone + Copy + Debug> IdTrait for T {}

#[derive(Clone)]
pub struct Forest<ID> {
    map: FxHashMap<ID, TreeNode<ID>>,
}

impl<ID: Hash + PartialEq + Eq> PartialEq for Forest<ID> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map
    }
}

impl<ID: Hash + PartialEq + Eq + Debug> Debug for Forest<ID> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Forest").field("map", &self.map).finish()
    }
}

impl<ID: Hash + PartialEq + Eq> Eq for Forest<ID> {}

/// Immutable tree node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TreeNode<ID> {
    pub(crate) parent: Option<ID>,
    pub(crate) deleted: bool,
}

#[derive(Debug, Clone)]
pub enum Error {
    CyclicMoveErr,
}

impl<ID: IdTrait> Forest<ID> {
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            map: Default::default(),
        }
    }

    /// Move node into new_parent.
    /// It will **create a new node** if node is not contained in the current map
    ///
    /// Return Err when the action will cause cycle in tree
    pub fn mov(&mut self, node_id: ID, parent_id: Option<ID>) -> Result<(), Error> {
        let mut deleted = false;
        let mut contained = false;
        if let Some(node) = self.map.get(&node_id) {
            contained = true;
            if node.deleted {
                deleted = true;
            }
        }

        // The current implementation doesn't preserve the hierarchy directly,
        // but it can be inferred.
        // So we cannot travel the forest cheaply. It needs O(n) to construct the trees first.
        if parent_id.is_none() {
            self.map.insert(
                node_id,
                TreeNode {
                    parent: None,
                    deleted,
                },
            );
            return Ok(());
        }

        let parent_id = parent_id.unwrap();
        assert!(
            self.map.contains_key(&parent_id),
            "Parent id {:?} does not exist.",
            parent_id
        );

        if contained {
            if self.is_ancestor_of(node_id, parent_id) {
                return Err(Error::CyclicMoveErr);
            }

            let node = self.map.get_mut(&node_id).unwrap();
            node.parent = Some(parent_id);
        } else {
            self.map.insert(
                node_id,
                TreeNode {
                    parent: Some(parent_id),
                    deleted: false,
                },
            );
        }

        Ok(())
    }

    #[inline(never)]
    fn is_ancestor_of(&self, maybe_ancestor: ID, node_id: ID) -> bool {
        if maybe_ancestor == node_id {
            return true;
        }

        let mut node_id = node_id;
        loop {
            let node = self.map.get(&node_id).unwrap();
            match node.parent {
                Some(parent_id) if parent_id == maybe_ancestor => return true,
                Some(parent_id) if parent_id == node_id => panic!("loop detected"),
                Some(parent_id) => {
                    node_id = parent_id;
                }
                None => return false,
            }
        }
    }

    pub fn delete(&mut self, node_id: ID) {
        self.map.get_mut(&node_id).unwrap().deleted = true;
    }

    pub fn undo_delete(&mut self, node_id: ID) {
        self.map.get_mut(&node_id).unwrap().deleted = false;
    }

    pub(crate) fn get(&self, id: &ID) -> Option<&TreeNode<ID>> {
        self.map.get(id)
    }
}

impl<ID: IdTrait> Default for Forest<ID> {
    fn default() -> Self {
        Self::new()
    }
}
