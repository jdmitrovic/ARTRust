use crate::{ ARTree, ARTKey };
use crate::node::{ ARTNode, ARTLeaf, ARTInnerNode };
use std::marker::PhantomData;
use std::rc::Rc;
use crate::keys::*;

impl<'a, K: ARTKey, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
            _marker: PhantomData,
        }
    }


    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let mut current = &mut self.root;
        let mut depth: usize = 0;

        while let Some(node) = current.take() {
            match *node {
                ARTNode::Leaf(leaf) => {
                    let mut new_inner: ARTInnerNode<K, V> = ARTInnerNode::new_inner_4(0);
                    new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                    let key_byte = leaf.key()[depth];
                    new_inner.add_node(Box::new(ARTNode::Leaf(leaf)), key_byte);
                    current.replace(Box::new(ARTNode::Inner(new_inner, Default::default())));
                    break;
                }
                ARTNode::Inner(mut inner, inner_pkey) => {
                    let pk_size = inner.partial_key_size();
                    let current_pkey = &key_bytes[depth..depth+ pk_size as usize + 1];

                    match compare_pkeys(&inner_pkey, current_pkey) {
                        PartialKeyComp::PartialMatch(len) => {
                            depth += len;
                            let mut new_inner: ARTInnerNode<K, V> = ARTInnerNode::new_inner_4(len as u8);
                            let key_byte = inner_pkey[depth];
                            new_inner.add_node(Box::new(ARTNode::Inner(inner, inner_pkey)), key_byte);
                            new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                            current.replace(Box::new(ARTNode::Inner(new_inner, Default::default())));
                            break;
                        }
                        PartialKeyComp::FullMatch(len) => {
                            depth = depth;
                            depth += len;
                            let next = inner.find_child_mut(current_pkey[depth]).unwrap();
                            current.replace(Box::new(ARTNode::Inner(inner, Default::default())));
                            current = next;
                        }
                    }
                }
            }
        }

        // if current.is_none() {
        //     current.replace(Box::new(ARTNode::Leaf(ARTLeaf::new(&key_bytes, value))));
        // }
    }


    // pub fn insert(&mut self, key: K, value: V) {
    //     let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
    //     let mut current_link = &mut self.root;
    //     let mut current = current_link.take();
    //     let mut depth: usize = 0;

    //     loop {
    //         let (next_link, key_insert) = match current {
    //             None => {
    //                 // current_link.replace(Box::new(ARTNode::Leaf(ARTLeaf::new(&key_bytes, value))));
    //                 // break;
    //             }
    //             Some(current_node) => {
    //                 match *current_node {
    //                     ARTNode::Leaf(_) => {
    //                         let mut new_inner = ARTInnerNode::new_inner_4(0);
    //                         // new_inner.add_node(current_node, key_bytes[depth]);
    //                         new_inner.add_child(&key_bytes, value, key_bytes[depth]);
    //                         (Box::new(ARTNode::Inner(new_inner, Rc::new(vec![]))),
    //                          Some(key_bytes[depth]))
    //                     }
    //                     ARTNode::Inner(inner_node, ref inner_pkey) => {
    //                         let pk_size = inner_node.partial_key_size();
    //                         let current_pkey = &key_bytes[depth..depth+ pk_size as usize + 1];

    //                         match compare_pkeys(inner_pkey, current_pkey) {
    //                             PartialKeyComp::PartialMatch(len) => {
    //                                 depth += len;
    //                                 let mut new_inner = ARTInnerNode::new_inner_4(len as u8);
    //                                 let byte_key = inner_pkey[depth];
    //                                 // new_inner.add_node(current_node, byte_key);
    //                                 new_inner.add_child(&key_bytes, value, key_bytes[depth]);
    //                                 (Box::new(ARTNode::Inner(new_inner,
    //                                                               Rc::new(current_pkey.to_vec()))),
    //                                  Some(byte_key))
    //                             }
    //                             PartialKeyComp::FullMatch(len) => {
    //                                 depth = depth;
    //                                 depth += len;
    //                                 let next = inner_node.find_child_mut(current_pkey[depth]).unwrap();
    //                                 if next.is_none() {
    //                                     inner_node.add_node(ARTNode::Leaf(ARTLeaf::new(&key_bytes, value)),
    //                                                         key_bytes[depth]);
    //                                     break;
    //                                 }
    //                             }
    //                         }
    //                     }
    //                 }
    //             }
    //         };

    //         match key_insert {
    //             None => {
    //                 current_link.replace(current.unwrap());
    //                 current_link = Some(&mut next_link);
    //                 current = current_link.take();
    //             }
    //             Some(key_byte) => {
    //                 // current_link
    //             }
    //         }

    //     }
    // }

    // pub fn find(&self, key: K) -> Option<&V> {
    //     let key_bytes = key.convert_to_bytes();
    //     let mut current = self.root.as_ref();
    //     let mut depth: usize = 0;

    //     loop {
    //         match current {
    //             None => break None,
    //             Some(node) => {
    //                 match **node {
    //                     ARTNode::Leaf(leaf) => {
    //                         if let LeafKeyComp::FullMatch = compare_leafkey(&leaf.key()[depth..],
    //                                                                         &key_bytes[depth..]) {
    //                             break Some(leaf.value());
    //                         } else {
    //                             break None;
    //                         }
    //                     }
    //                     ARTNode::Inner(inner_node, pkey) => {
    //                         let pkey_size = inner_node.partial_key_size();
    //                         match key_bytes.get(depth..depth + pkey_size as usize) {
    //                             None => break None,
    //                             Some(pkey_2) => {
    //                                 match compare_pkeys(&pkey, pkey_2) {
    //                                     PartialKeyComp::PartialMatch(_) => break None,
    //                                     PartialKeyComp::FullMatch(len) => {
    //                                         depth += len;
    //                                         current = inner_node.find_child(key_bytes[depth]).as_ref();
    //                                     }
    //                                 }
    //                             }
    //                         }

    //                     }
    //                 }
    //             }
    //         }
    //     }
    // }
}

