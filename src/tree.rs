use crate::{ ARTree, ARTKey };
use crate::node::{ ARTNode, ARTLeaf, ARTInnerNode };
use std::marker::PhantomData;
use std::rc::Rc;
use crate::keys::*;

impl<K: ARTKey, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
            _marker: PhantomData,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let current_link = &mut self.root;
        let mut depth: usize = 0;

        loop {
            let current = current_link.take();
            match current {
                None => {
                    current_link.replace(ARTNode::Leaf(Box::new(ARTLeaf::new(&key_bytes, value))));
                    break;
                }
                Some(mut current_node) => {
                    match current_node {
                        ARTNode::Leaf(_) => {
                            let mut new_inner = ARTInnerNode::new_inner_4(0);
                            new_inner.add_node(current_node, key_bytes[depth]);
                            new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                            current_link.replace(ARTNode::Inner(new_inner, Rc::new(vec![])));
                            break;
                        }
                        ARTNode::Inner(ref mut inner_node, ref inner_pkey) => {
                            let pk_size = inner_node.partial_key_size();
                            let current_pkey = &key_bytes[depth..depth+ pk_size as usize + 1];

                            match compare_pkeys(inner_pkey, current_pkey) {
                                PartialKeyComp::PartialMatch(len) => {
                                    depth += len;
                                    let mut new_inner = ARTInnerNode::new_inner_4(len as u8);
                                    let byte_key = inner_pkey[depth];
                                    new_inner.add_node(current_node, byte_key);
                                    new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                                    current_link.replace(ARTNode::Inner(new_inner,
                                                                        Rc::new(current_pkey.to_vec())));
                                    break;
                                }
                                PartialKeyComp::FullMatch(len) => {
                                    depth += len;
                                    *current_link = inner_node.find_child_mut(current_pkey[depth]);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn find(&self, key: K) -> Option<&V> {
        let key_bytes = key.convert_to_bytes();
        let mut current = self.root.as_ref();
        let mut depth: usize = 0;

        loop {
            match current {
                None => break None,
                Some(node) => {
                    match node {
                        ARTNode::Leaf(leaf) => {
                            if let LeafKeyComp::FullMatch = compare_leafkey(&leaf.key()[depth..],
                                                                            &key_bytes[depth..]) {
                                break Some(leaf.value());
                            } else {
                                break None;
                            }
                        }
                        ARTNode::Inner(inner_node, pkey) => {
                            let pkey_size = inner_node.partial_key_size();
                            match key_bytes.get(depth..depth + pkey_size as usize) {
                                None => break None,
                                Some(pkey_2) => {
                                    match compare_pkeys(&pkey, pkey_2) {
                                        PartialKeyComp::PartialMatch(_) => break None,
                                        PartialKeyComp::FullMatch(len) => {
                                            depth += len;
                                            current = inner_node.find_child(key_bytes[depth]);
                                        }
                                    }
                                }
                            }

                        }
                    }
                }
            }
        }
    }
}

