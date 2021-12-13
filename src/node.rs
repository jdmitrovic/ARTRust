use std::{marker::PhantomData, mem::{ self, MaybeUninit }};
use std::rc::Rc;

use crate::ARTKey;
use crate::keys::ByteKey;

pub enum ARTNode<K: ARTKey, V> {
    Inner(ARTInnerNode<K, V>, ByteKey),
    Leaf(Box<ARTLeaf<K, V>>),
}

pub type ARTLink<K, V> = Option<ARTNode<K, V>>;

pub enum ARTInnerNode<K: ARTKey, V> {
    Inner4(Box<ARTInner4<K, V>>),
    Inner256(Box<ARTInner256<K, V>>),
}

pub struct ARTInner4<K: ARTKey, V> {
    pkey_size: u8,
    keys: [Option<u8>; 4],
    children: [ARTLink<K, V>; 4],
}

pub struct ARTInner256<K: ARTKey, V> {
    pkey_size: u8,
    children: [ARTLink<K, V>; 256],
}

pub struct ARTLeaf<K: ARTKey, V>{
    key: ByteKey,
    value: V,
    _marker: PhantomData<K>,
}

impl<K: ARTKey, V> ARTLeaf<K, V> {
    pub fn new(byte_key: &ByteKey, value: V) -> Self {
        ARTLeaf {
            key: Rc::clone(byte_key),
            value,
            _marker: PhantomData,
        }
    }

    pub fn new_bytekey(key: &ByteKey, value: V) -> Self {
        ARTLeaf {
            key: Rc::clone(key),
            value,
            _marker: PhantomData,
        }
    }
}

impl<K: ARTKey, V> ARTInnerNode<K, V> {
    pub fn new_inner_4(key: &ByteKey, pkey_size: u8) -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner4(Box::new(ARTInner4 {
            keys: Default::default(),
            children: Default::default(),
            pkey_size
        }))
    }

    pub fn new_inner_256(pkey: &ByteKey, pkey_size: u8) -> ARTInnerNode<K, V> {
        let children = {
            let mut arr: [MaybeUninit<Option<ARTNode<K, V>>>; 256] = unsafe {
                MaybeUninit::uninit().assume_init()
            };

            for item in &mut arr[..] {
                item.write(None);
            }

            unsafe { mem::transmute::<_, [Option<ARTNode<K, V>>; 256]>(arr) }
        };

        ARTInnerNode::Inner256(Box::new(ARTInner256 {
            children,
            pkey_size,
        }))
    }

    fn partial_key_size(&self) -> u8 {
        match self {
            ARTInnerNode::Inner4(node) => node.pkey_size,
            ARTInnerNode::Inner256(node) => node.pkey_size,
        }
    }

    pub fn add_child(&mut self, byte_key: &ByteKey, value: V, key_byte: u8) {
        self.add_node(ARTNode::Leaf(Box::new(ARTLeaf::new(byte_key, value))), key_byte)
    }

    pub fn add_node(&mut self, new_node: ARTNode<K, V>, key_byte: u8) {
        match self {
            ARTInnerNode::Inner4(node) => {
                for (i, key) in node.keys.iter_mut().enumerate() {
                    match key {
                        Some(_) => continue,
                        None => {
                            key.replace(key_byte);
                            node.children[i].replace(new_node);
                            return;
                        }
                    }
                }

               // node.grow();
            }
            ARTInnerNode::Inner256(node) => {
                node.children[key_byte as usize].replace(new_node).unwrap();
            }
        }
    }

    pub fn iter(&self, key: K) -> Iter<K, V> {
        Iter {
            next: Some(self),
            key_bytes: key.convert_to_bytes(),
            depth: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let mut current_link = Some(self);
        let mut depth: usize = 0;

        loop {
            let current = current_link.take();
            match current {
                None => current_link = current,
                Some(current_node) => {
                    match current_node {
                        ARTNode::Leaf(leaf) => {

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

        current.add_child(&key_bytes, value, key_bytes[depth]);
    }


    fn find_child(&mut self, key_byte: u8) -> Option<&mut ARTLink<K, V>> {
        match self {
            ARTInnerNode::Inner4(node) => {
                let pos = node.keys.iter().position(|k| {
                    match k {
                        None => false,
                        Some(x) => *x == key_byte,
                    }
                });

                pos.map(move |i| &mut node.children[i])
            }

            ARTInnerNode::Inner256(node) => Some(&mut node.children[key_byte as usize]),
        }
    }
}

impl<K: ARTKey, V> ARTInner4<K, V> {
    // fn grow(&mut self) -> ARTNode<K, V> {
    //     ARTNode::Inner(ARTInnerNode::new_inner_256(&self.keys, self.pkey_size))
    // }
}

pub struct Iter<'a, K: ARTKey, V> {
    next: Option<&'a ARTInnerNode<K, V>>,
    key_bytes: Vec<u8>,
    depth: u32,
}

impl<'a, K: ARTKey, V> Iterator for Iter<'a, K, V> {
    type Item = &'a ARTInnerNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|inner_node| {
            let potential_next = match inner_node {
                ARTInnerNode::Inner4(node) => {
                    self.depth += node.pkey_size as u32;

                    let key_byte = self.key_bytes[self.depth as usize];
                    let pos = node.keys.iter().position(|k| {
                        match k {
                            None => false,
                            Some(x) => *x == key_byte,
                        }
                    });

                    pos.map(|i| {
                        self.depth += 1;
                        &node.children[i]
                    }).expect("There should be a node in this position")
                }

                ARTInnerNode::Inner256(node) => {
                    let key_byte: u8 = self.key_bytes[self.depth as usize];
                    self.depth += 1;

                    &node.children[key_byte as usize]
                }
            };

            if let Some(ARTNode::Inner(ref pn_inner, partial_key)) = potential_next {
                let pk = &self.key_bytes[self.depth as usize..partial_key.len()];
                if partial_key.iter().zip(pk).all(|(a, b)| a == b) == false {
                    self.next = None;
                } else {
                    self.next = Some(pn_inner);
                }
            } else {
                self.next = None;
            }

           inner_node
        })
    }
}

impl<K: ARTKey, V> ARTNode<K, V> {
    pub fn insert(&mut self, key: K, value: V) {
        let key_bytes: ByteKey = Rc::new(key.convert_to_bytes());
        let mut current_link = Some(self);
        let mut depth: usize = 0;

        loop {
            let current = current_link.take();
            match current {
                None => current_link = current,
                Some(current_node) => {
                    match current_node {
                        ARTNode::Leaf(leaf) => {

                        }
                        ARTNode::Inner(inner_node, pkey) => {
                            let pk_size = inner_node.partial_key_size();
                            let current_pkey = &key_bytes[depth..depth+ pk_size as usize];
                            let pkey_match_size = compare_pkeys(pkey, current_pkey);

                            if pkey_match_size
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

        current.add_child(&key_bytes, value, key_bytes[depth]);
    }
}

pub fn compare_pkeys(pkey_1: &[u8], pkey_2: &[u8]) -> usize {
        match pkey_1.iter().zip(pkey_2.iter()).position(|(a, b)| a != b) {
            None => std::cmp::min(pkey_1.len(), pkey_2.len()),
            Some(pos) => pos,
        }
    }
