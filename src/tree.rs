use crate::{ ARTree, ARTKey };
use crate::node::{ ARTLink, ARTNode, ARTLeaf, ARTInnerNode };
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


    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let key_len = key_bytes.len();
        let mut current_link = &mut self.root;
        let mut depth: usize = 0;
        let mut inner_byte: u8 = Default::default();
        let mut pkey_len: usize = 0;
        let mut partial_match = false;

        while let Some(box ARTNode::Inner(ref mut inner, ref pkey, ref mut val)) = current_link {
            let pk_size = inner.partial_key_size();
            let current_pkey = &key_bytes.get(depth..depth + pk_size as usize)
                                         .unwrap_or(&key_bytes[depth..]);

            match compare_pkeys(pkey, current_pkey) {
                PartialKeyComp::FullMatch(len) => {
                    depth += len;
                    if depth == key_len {
                        return val.replace(value);
                    }
                    match inner.find_child_mut(key_bytes[depth]) {
                        None => break,
                        Some(link) => {
                            current_link = unsafe { &mut *link };
                        }
                    }

                    depth += 1;
                }
                PartialKeyComp::PartialMatch(len) => {
                    depth += len;
                    pkey_len = len;
                    inner_byte = pkey[depth];
                    inner.reduce_pkey_size(len as u8);
                    partial_match = true;
                    break;
                }
            }
        }

        match current_link.take() {
            None => {
                current_link.replace(Box::new(ARTNode::Leaf(ARTLeaf::new(&key_bytes, value))));
                None
            }
            Some(mut node) => {
                if partial_match {
                    let mut new_inner = ARTInnerNode::new_inner_4(pkey_len as u8);
                    new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                    new_inner.add_node(node, inner_byte);
                    current_link.replace(Box::new(ARTNode::Inner(new_inner,
                                                                Rc::clone(&key_bytes),
                                                                None)));
                    return None;
                }

                match *node {
                    ARTNode::Inner(inner, pkey, val) => {
                        let mut inner = if inner.is_full() {
                            inner.grow()
                        } else {
                            inner
                        };

                        inner.add_child(&key_bytes, value, key_bytes[depth]);
                        current_link.replace(Box::new(ARTNode::Inner(inner,
                                                                     pkey,
                                                                     val)));
                        None
                    }
                    ARTNode::Leaf(ref mut leaf) => {
                        match compare_leafkeys(&leaf.key()[depth..], &key_bytes[depth..]) {
                            PartialKeyComp::FullMatch(_) => {
                                let ret = leaf.change_value(value);
                                current_link.replace(node);
                                Some(ret)
                            }
                            PartialKeyComp::PartialMatch(_) => {
                                let mut new_inner = ARTInnerNode::new_inner_4(0);
                                // dbg!(key_bytes[depth] as char);
                                new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                                let byte: u8 = leaf.key().get(depth).unwrap().to_owned();

                                // dbg!(byte as char);

                                new_inner.add_node(node, byte);
                                current_link.replace(Box::new(ARTNode::Inner(new_inner,
                                                                            Rc::clone(&key_bytes),
                                                                            None)));
                                None
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let key_len = key_bytes.len();
        let mut current_link = &mut self.root;
        let mut depth: usize = 0;

        while let Some(box ARTNode::Inner(ref mut inner, ref pkey, ref mut val)) = current_link {
            let pk_size = inner.partial_key_size();
            let current_pkey = &key_bytes.get(depth..depth + pk_size as usize)
                                         .unwrap_or(&key_bytes[depth..]);

            match compare_pkeys(pkey, current_pkey) {
                PartialKeyComp::FullMatch(len) => {
                    depth += len;
                    if depth == key_len {
                        // key match in inner node
                        break;
                    }
                    match inner.find_child_mut(key_bytes[depth]) {
                        None => return None,
                        Some(link) => {
                            let new_link = unsafe { &mut *link };

                            if let Some(box ARTNode::Leaf(ref leaf)) = new_link {
                                match compare_leafkeys(&leaf.key()[depth..], &key_bytes[depth..]) {
                                    PartialKeyComp::FullMatch(_) => break,
                                    PartialKeyComp::PartialMatch(_) => return None,
                                }
                            } else {
                                current_link = new_link;
                                depth += 1;
                            }
                        }
                    }

                }
                PartialKeyComp::PartialMatch(_) => {
                    return None;
                }
            }
        }

        match current_link.take() {
            None => return None,
            Some(node) => {
                match *node {
                    ARTNode::Inner(mut inner, pkey, val) => {
                        if depth == key_len {
                            // shrink needed
                            current_link.replace(Box::new(ARTNode::Inner(inner, pkey, None)));
                            return val;
                        }

                        let former_val = inner.as_mut().remove_child(key_bytes[depth]);
                        current_link.replace(Box::new(ARTNode::Inner(inner, pkey, val)));
                        former_val
                    }
                    ARTNode::Leaf(leaf) => {
                        // only if tree consists only of one leaf node
                        match compare_leafkeys(&leaf.key()[depth..], &key_bytes[depth..]) {
                            PartialKeyComp::FullMatch(_) => {
                                return Some(leaf.take_value());
                            }
                            PartialKeyComp::PartialMatch(_) => {
                                current_link.replace(Box::new(ARTNode::Leaf(leaf)));
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn find(&self, key: K) -> Option<&V> {
        let key_bytes = key.convert_to_bytes();
        let key_len = key_bytes.len();
        let mut current = self.root.as_ref();
        let mut depth: usize = 0;

        loop {
            match current {
                None => break None,
                Some(node) => {
                    match **node {
                        ARTNode::Leaf(ref leaf) => {
                            if let PartialKeyComp::FullMatch(_) = compare_leafkeys(&leaf.key()[depth..],
                                                                             &key_bytes[depth..]) {
                                break Some(leaf.value());
                            } else {
                                break None;
                            }
                        }

                        ARTNode::Inner(ref inner_node, ref pkey, ref val) => {
                            let pkey_size = inner_node.partial_key_size();
                            match key_bytes.get(depth..depth + pkey_size as usize) {
                                None => break None,
                                Some(pkey_2) => {
                                    match compare_pkeys(&pkey, pkey_2) {
                                        PartialKeyComp::PartialMatch(_) => break None,
                                        PartialKeyComp::FullMatch(len) => {
                                            depth += len;
                                            if depth == key_len {
                                                break val.as_ref();
                                            }

                                            current = inner_node.find_child(key_bytes[depth]);
                                            depth += 1;
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

