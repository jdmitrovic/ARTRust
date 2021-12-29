use std::{marker::PhantomData, mem::{ self, MaybeUninit }};
use std::rc::Rc;

use crate::ARTKey;
use crate::keys::{*};

pub enum ARTNode<K: ARTKey, V> {
    Inner(Box<ARTInnerNode<K, V>>, ByteKey),
    Leaf(ARTLeaf<K, V>),
}

pub type ARTLink<K, V> = Option<Box<ARTNode<K, V>>>;

pub enum ARTInnerNode<K: ARTKey, V> {
    Inner4(ARTInner4<K, V>),
    Inner256(ARTInner256<K, V>),
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

    pub fn key(&self) -> &[u8] {
        &self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }
}

impl<'a, K: ARTKey, V> ARTInnerNode<K, V> {
    pub fn new_inner_4(pkey_size: u8) -> Box<ARTInnerNode<K, V>> {
        Box::new(ARTInnerNode::Inner4(ARTInner4 {
            keys: Default::default(),
            children: Default::default(),
            pkey_size
        }))
    }

    pub fn new_inner_256(pkey_size: u8) -> ARTInnerNode<K, V> {
        let children = {
            let mut arr: [MaybeUninit<Option<Box<ARTNode<K, V>>>>; 256] = unsafe {
                MaybeUninit::uninit().assume_init()
            };

            for item in &mut arr[..] {
                item.write(None);
            }

            unsafe { mem::transmute::<_, [Option<Box<ARTNode<K, V>>>; 256]>(arr) }
        };

        ARTInnerNode::Inner256(ARTInner256 {
            children,
            pkey_size,
        })
    }

    pub fn partial_key_size(&self) -> u8 {
        match self {
            ARTInnerNode::Inner4(node) => node.pkey_size,
            ARTInnerNode::Inner256(node) => node.pkey_size,
        }
    }

    pub fn add_child(&mut self, byte_key: &ByteKey, value: V, key_byte: u8) {
        self.add_node(Box::new(ARTNode::Leaf(ARTLeaf::new(byte_key, value))), key_byte)
    }

    pub fn add_node(&mut self, new_node: Box<ARTNode<K, V>>, key_byte: u8) {
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

    pub fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<K, V>> {
        match self {
            ARTInnerNode::Inner4(node) => {
                let pos = node.keys.iter().position(|k| {
                    match k {
                        None => false,
                        Some(x) => *x == key_byte,
                    }
                });

                pos.map(move |i| &mut node.children[i] as *mut ARTLink<K, V>)
            }

            ARTInnerNode::Inner256(node) => Some(&mut node.children[key_byte as usize] as *mut ARTLink<K, V>),
        }
    }

    pub fn find_child(&self, key_byte: u8) -> Option<&Box<ARTNode<K, V>>> {
        match self {
            ARTInnerNode::Inner4(node) => {
                let pos = node.keys.iter().position(|k| {
                    match k {
                        None => false,
                        Some(x) => *x == key_byte,
                    }
                });

                pos.map(move |i| node.children[i].as_ref()).flatten()
            }

            ARTInnerNode::Inner256(node) => node.children[key_byte as usize].as_ref(),
        }
    }
}

impl<K: ARTKey, V> ARTInner4<K, V> {
    // fn grow(&mut self) -> ARTNode<K, V> {
    //     ARTNode::Inner(ARTInnerNode::new_inner_256(&self.keys, self.pkey_size))
    // }
}
