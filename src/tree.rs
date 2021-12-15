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

    // pub fn insert(&mut self, key: K, value: V) {
    //     let mut old_root = self.root.take();

    //     match old_root {
    //         None => {
    //             let byte_key = Rc::new(key.convert_to_bytes());
    //             self.root.replace(ARTNode::Leaf(Box::new(ARTLeaf::new(&byte_key, value))));
    //         }
    //         Some(ref mut node) => {
    //             match node {
    //                 ARTNode::Leaf(leaf) => {
    //                     // let mut new_root = ARTInnerNode::<K, V>::new_inner_4();
    //                     // let old_key_byte = leaf.key().convert_to_bytes()[0];
    //                     // new_root.add_node(old_root.unwrap(), old_key_byte);
    //                     // new_root.add_child(key, value, key_bytes[0]);
    //                     // self.root.replace(ARTNode::Inner(new_root));
    //                 }
    //                 ARTNode::Inner(inner, _) => {
    //                     // let inner_iter = inner.iter_mut(&key);
    //                     // let key_byte = inner_iter.key_byte();
    //                     // inner_iter.last().unwrap().add_child(key, value, key_byte);
    //                 }
    //             }
    //         }
    //     };
    // }


    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let current_link = &mut self.root;
        let mut depth: usize = 0;

        loop {
            let current = current_link.take();
            match current {
                None => { current_link.replace(ARTNode::Leaf(Box::new(ARTLeaf::new(&key_bytes, value)))); },
                Some(current_node) => {
                    match current_node {
                        ARTNode::Leaf(_) => {
                            let mut new_inner = ARTInnerNode::new_inner_4(&key_bytes, 0);
                            new_inner.add_node(current_node, key_bytes[depth]);
                            new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                            current_link.replace(ARTNode::Inner(new_inner, Rc::new(vec![])));
                            break;
                        }
                        ARTNode::Inner(mut inner_node, inner_pkey) => {
                            let pk_size = inner_node.partial_key_size();
                            let current_pkey = &key_bytes[depth..depth+ pk_size as usize];

                            match compare_pkeys(&inner_pkey, current_pkey) {
                                PartialKeyComp::PartialMatch(len) => {
                                    depth += len;
                                    let mut new_inner = ARTInnerNode::new_inner_4(&inner_pkey, len as u8);
                                    // new_inner.add_node(current.take().unwrap(), inner_pkey[depth]);
                                    new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                                    current_link.replace(ARTNode::Inner(new_inner,
                                                                        Rc::new(current_pkey.to_vec())));
                                }
                                PartialKeyComp::FullMatch(len) => {
                                    *current_link = inner_node.find_child(inner_pkey[len + 1]);
                                }
                            }
                        }
                    }
                }
            }

            // let pk_size = current.partial_key_size();
            // let potential_next = current.find_child(key_bytes[depth]);
            // depth += 1;

            // if let None = potential_next {
            //     break;
            // }

            // let potential_next = potential_next.unwrap();

            // if let Some(ARTNode::Inner(pn_inner, pk_inner)) = potential_next {
            //     let pk_leaf = &key_bytes[depth..depth+ pk_size as usize];
            //     match pk_inner.iter().zip(pk_leaf).position(|(a, b)| a != b) {
            //         None => {
            //             current = pn_inner;
            //             depth += pk_inner.len();
            //             continue;
            //         }
            //         Some(pos) => {
            //             let mut new_inner = ARTInnerNode::new_inner_4(pk_inner, pos as u8);
            //             new_inner.add_node(potential_next.take().unwrap(), pk_inner[depth]);
            //             potential_next.replace(ARTNode::Inner(new_inner, Rc::clone(pk_inner)));
            //         }
            //     }
            // }

            break;
        }

        // current.add_child(&key_bytes, value, key_bytes[depth]);
    }
}

