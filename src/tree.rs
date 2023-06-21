use im::HashMap as ImHashMap;
use std::{fmt::Debug, hash::Hash};

pub trait IdTrait: Hash + Eq + Clone + Copy + Debug {}
impl<T: Hash + Eq + Clone + Copy + Debug> IdTrait for T {}

///
///
#[derive(Clone)]
pub struct Forest<ID> {
    map: ImHashMap<ID, TreeNode<ID>>,
}

/// Immutable tree node.
#[derive(Debug, Clone, Copy)]
struct TreeNode<ID> {
    pub(crate) parent: Option<ID>,
}

#[derive(Debug, Clone)]
pub struct CyclicMoveErr;

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
    pub fn mov(&mut self, node_id: ID, parent_id: Option<ID>) -> Result<(), CyclicMoveErr> {
        // The current implementation doesn't preserve the hierarchy directly,
        // but it can be inferred.
        // So we cannot travel the forest cheaply. It needs O(n) to construct the trees first.
        if parent_id.is_none() {
            self.map.insert(node_id, TreeNode { parent: None });
            return Ok(());
        }

        let parent_id = parent_id.unwrap();
        assert!(
            self.map.contains_key(&parent_id),
            "Parent id {:?} does not exist.",
            parent_id
        );
        if self.map.contains_key(&node_id) {
            if self.is_ancestor_of(node_id, parent_id) {
                return Err(CyclicMoveErr);
            }

            let node = self.map.get_mut(&node_id).unwrap();
            node.parent = Some(parent_id);
        } else {
            self.map.insert(
                node_id,
                TreeNode {
                    parent: Some(parent_id),
                },
            );
        }

        Ok(())
    }

    fn is_ancestor_of(&self, maybe_ancestor: ID, node_id: ID) -> bool {
        let mut node_id = node_id;
        loop {
            let node = self.map.get(&node_id).unwrap();
            match node.parent {
                Some(parent_id) if parent_id == maybe_ancestor => return true,
                Some(parent_id) => {
                    node_id = parent_id;
                }
                None => return false,
            }
        }
    }
}

impl<ID: IdTrait> Default for Forest<ID> {
    fn default() -> Self {
        Self::new()
    }
}
