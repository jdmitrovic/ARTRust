use crate::keys::{compare_leaf_keys, compare_pkeys, ARTKey, LeafKeyComp, PartialKeyComp};
use crate::node::{ARTInnerNode, ARTLeaf, ARTNode, InnerNode};
use crate::ARTree;
use std::rc::Rc;

impl<K: ARTKey, V> Default for ARTree<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K: ARTKey, V> ARTree<K, V> {
    pub fn new() -> Self {
        ARTree {
            root: None,
            _marker: Default::default(),
        }
    }

    pub fn insert_or_update(&mut self, key: K, value: V) -> Option<V> {
        let key_bytes = key.into_byte_key();
        let key_len = key_bytes.len();
        let mut current_link = &mut self.root;
        let mut depth: usize = 0;
        let mut inner_byte: u8 = Default::default();
        let mut pkey_len: usize = 0;
        let mut partial_match = false;

        while let Some(ARTNode::Inner(ref mut inner, ref pkey, ref mut val)) = current_link {
            let pk_size = inner.partial_key_size();
            let end = (depth + pk_size as usize).min(key_bytes.len());
            let current_pkey = &key_bytes[depth..end];

            match compare_pkeys(pkey, current_pkey) {
                PartialKeyComp::FullMatch(len) => {
                    depth += len;
                    if depth == key_len {
                        return val.replace(value);
                    }
                    if let Some(link) = inner.find_child_mut(key_bytes[depth]) {
                        current_link = unsafe { &mut *link };
                    } else {
                        break;
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

        if let Some(node) = current_link.take() {
            if partial_match {
                let mut new_inner = ARTInnerNode::new_inner_4(pkey_len as u8);
                new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                new_inner.add_node(node, inner_byte);
                current_link.replace(ARTNode::Inner(new_inner, Rc::clone(&key_bytes), None));
                return None;
            }

            match node {
                ARTNode::Inner(inner, pkey, val) => {
                    let mut inner = if inner.is_full() { inner.grow() } else { inner };

                    inner.add_child(&key_bytes, value, key_bytes[depth]);
                    *current_link = Some(ARTNode::Inner(inner, pkey, val));
                }
                ARTNode::Leaf(mut leaf) => {
                    match compare_leaf_keys(&leaf.key()[depth..], &key_bytes[depth..]) {
                        LeafKeyComp::FullMatch => {
                            let ret = leaf.change_value(value);
                            *current_link = Some(ARTNode::Leaf(leaf));
                            return Some(ret);
                        }
                        LeafKeyComp::PartialMatch(len) => {
                            depth += len;
                            let mut new_inner = ARTInnerNode::new_inner_4(len as u8);
                            new_inner.add_child(&key_bytes, value, key_bytes[depth]);
                            let byte: u8 = leaf.key()[depth];

                            new_inner.add_node(ARTNode::Leaf(leaf), byte);
                            *current_link =
                                Some(ARTNode::Inner(new_inner, Rc::clone(&key_bytes), None));
                        }
                        LeafKeyComp::CompleteMatchLeft(len) => {
                            depth += len;
                            let mut new_inner = ARTInnerNode::new_inner_4(len as u8);
                            new_inner.add_child(&key_bytes, value, key_bytes[depth]);

                            *current_link = Some(ARTNode::Inner(
                                new_inner,
                                Rc::clone(&key_bytes),
                                Some(leaf.take_value()),
                            ));
                        }
                        LeafKeyComp::CompleteMatchRight(len) => {
                            depth += len;
                            let mut new_inner = ARTInnerNode::new_inner_4(len as u8);
                            let byte: u8 = leaf.key()[depth];

                            new_inner.add_node(ARTNode::Leaf(leaf), byte);
                            *current_link = Some(ARTNode::Inner(
                                new_inner,
                                Rc::clone(&key_bytes),
                                Some(value),
                            ));
                        }
                    }
                }
            }
        } else {
            *current_link = Some(ARTNode::Leaf(ARTLeaf::new(&key_bytes, value)));
        }
        None
    }

    pub fn delete(&mut self, key: K) -> Option<V> {
        let key_bytes = key.into_byte_key();
        let key_len = key_bytes.len();
        let mut current_link = &mut self.root;
        let mut depth: usize = 0;

        while let Some(ARTNode::Inner(ref mut inner, ref pkey, _)) = current_link {
            let pk_size = inner.partial_key_size();
            let end = (depth + pk_size as usize).min(key_bytes.len());
            let current_pkey = &key_bytes[depth..end];

            if let PartialKeyComp::FullMatch(len) = compare_pkeys(pkey, current_pkey) {
                depth += len;
                if depth == key_len {
                    // key match in inner node
                    break;
                }

                let link = inner.find_child_mut(key_bytes[depth])?;
                let new_link = unsafe { &mut *link };

                if let Some(ARTNode::Leaf(ref leaf)) = new_link {
                    if let LeafKeyComp::FullMatch =
                        compare_leaf_keys(&leaf.key()[depth..], &key_bytes[depth..])
                    {
                        break;
                    } else {
                        return None;
                    }
                } else {
                    current_link = new_link;
                    depth += 1;
                }
            } else {
                return None;
            }
        }

        match current_link.take()? {
            ARTNode::Inner(mut inner, pkey, val) => {
                if depth == key_len {
                    // shrink needed?
                    *current_link = Some(ARTNode::Inner(inner, pkey, None));
                    return val;
                }

                let former_val = inner.remove_child(key_bytes[depth]);
                *current_link = Some(ARTNode::Inner(inner, pkey, val));
                former_val
            }
            ARTNode::Leaf(leaf) => {
                // only if tree consists only of one leaf node
                if let LeafKeyComp::FullMatch =
                    compare_leaf_keys(&leaf.key()[depth..], &key_bytes[depth..])
                {
                    Some(leaf.take_value())
                } else {
                    *current_link = Some(ARTNode::Leaf(leaf));
                    None
                }
            }
        }
    }

    pub fn find(&self, key: K) -> Option<&V> {
        let key_bytes = key.convert_to_bytes();
        let key_bytes = key_bytes.as_ref();
        let key_len = key_bytes.len();
        let mut current = self.root.as_ref();
        let mut depth: usize = 0;

        while let Some(node) = current {
            match node {
                ARTNode::Leaf(leaf) => {
                    if let LeafKeyComp::FullMatch =
                        compare_leaf_keys(&leaf.key()[depth..], &key_bytes[depth..])
                    {
                        return Some(leaf.value());
                    } else {
                        return None;
                    }
                }

                ARTNode::Inner(inner_node, pkey, val) => {
                    let pkey_size = inner_node.partial_key_size();
                    let pkey_2 = key_bytes.get(depth..depth + pkey_size as usize)?;
                    if let PartialKeyComp::FullMatch(len) = compare_pkeys(pkey, pkey_2) {
                        depth += len;
                        if depth == key_len {
                            return val.as_ref();
                        }

                        current = inner_node.find_child(key_bytes[depth]);
                        depth += 1;
                    } else {
                        return None;
                    }
                }
            }
        }
        None
    }
}
