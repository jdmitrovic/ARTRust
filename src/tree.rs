use crate::{ ARTree, ARTKey };
use crate::node::{ ARTLink, ARTNode, ARTLeaf, ARTInnerNode};

impl ARTree {
    pub fn new() -> Self {
        ARTree {
            root: None,
        }
    }

    pub fn insert<K: ARTKey, V>(&mut self, key: K, value: V) {
        match self.root {
            None => self.root.insert(Box::new(ARTNode::Leaf(ARTLeaf::new(key, value)))),
            Some(node) => {
                match node {
                    ARTNode::Leaf(_) => {
                        let old_root = self.root.replace(ARTInnerNode::<K, V>::new_inner_4());
                        return
                    }
                    ARTNode::Inner(node) => {}
                }
            }
        }
    }

}
