use crate::{ ARTree, ARTKey };
use crate::node::{ ARTNode, ARTLeaf, ARTInnerNode };
use std::marker::PhantomData;
use std::rc::Rc;

impl<K: ARTKey, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut old_root = self.root.take();

        match old_root {
            None => {
                let byte_key = Rc::new(key.convert_to_bytes());
                self.root.replace(ARTNode::Leaf(Box::new(ARTLeaf::new(&byte_key, value))));
            }
            Some(ref mut node) => {
                match node {
                    ARTNode::Leaf(leaf) => {
                        // let mut new_root = ARTInnerNode::<K, V>::new_inner_4();
                        // let old_key_byte = leaf.key().convert_to_bytes()[0];
                        // new_root.add_node(old_root.unwrap(), old_key_byte);
                        // new_root.add_child(key, value, key_bytes[0]);
                        // self.root.replace(ARTNode::Inner(new_root));
                    }
                    ARTNode::Inner(inner, _) => {
                        // let inner_iter = inner.iter_mut(&key);
                        // let key_byte = inner_iter.key_byte();
                        // inner_iter.last().unwrap().add_child(key, value, key_byte);
                    }
                }
            }
        };
    }

}
