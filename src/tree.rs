use crate::{ ARTree, ARTKey };
use crate::node::{ ARTLink, ARTNode, ARTLeaf };

use std::mem;

impl ARTree {
    pub fn new() -> Self {
        ARTree {
            root: None,
        }
    }

    pub fn insert<K: ARTKey, V>(&mut self, key: K, value: V) {
        match self.root {
            None => mem::replace(&mut self.root, Some(ARTLeaf::new(key, value))),
            Some(node) => {}
        }
    }
}
