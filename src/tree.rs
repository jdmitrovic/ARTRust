use crate::{ ARTree, ARTKey };
use crate::node::{ ARTNode, ARTLeaf, ARTInnerNode };

impl<K: ARTKey, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let old_root = self.root.take();

        match old_root {
            None => {
                self.root.replace(ARTNode::Leaf(Box::new(ARTLeaf::new(key, value))));
            }
            Some(node) => {
                let key_bytes = key.convert_to_bytes();

                match node {
                    ARTNode::Leaf(ref leaf) => {
                        let mut new_root = ARTInnerNode::<K, V>::new_inner_4();
                        let old_key_byte = leaf.key().convert_to_bytes()[0];
                        new_root.add_node(node, old_key_byte);
                        new_root.add_child(key, value, key_bytes[0]);
                        self.root.replace(ARTNode::Inner(new_root));
                    }
                    ARTNode::Inner(_) => {
                    }
                }
            }
        };
    }

}
