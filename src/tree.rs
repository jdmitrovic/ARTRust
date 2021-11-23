use crate::{ ARTree, ARTKey };
use crate::node::{ ARTNode, ARTLeaf, ARTInnerNode };

impl<K, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.root {
            None => self.root.insert(Box::new(ARTNode::Leaf(ARTLeaf::new(key, value)))),
            Some(mut node) => {
                let key_bytes = key.convert_to_bytes();

                match node {
                    ARTNode::Leaf(_) => {
                        let new_root = ARTInnerNode::<K, V>::new_inner_4();
                        let old_root = self.root.take();
                        new_root.add_node(old_root, old_root.byte_key);
                        new_root.add_child(key, value, key_bytes[0]);
                        self.root.insert(Box::new(ARTNode::Inner(new_root)));
                    }
                    ARTNode::Inner(node) => {
                    }
                }
            }
        }
    }

}
