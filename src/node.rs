use std::{ marker::PhantomData, mem::{ self, MaybeUninit } };
use std::rc::Rc;

use crunchy::{self, unroll};
use std::arch::x86_64::_mm_extract_epi8;
use std::arch::x86_64::_mm_set1_epi8;
use std::arch::x86_64::_mm_cmpeq_epi8;
use std::arch::x86_64::_mm_movemask_epi8;
use std::arch::x86_64::_mm_xor_si128;
use std::arch::x86_64::_mm_slli_si128;
use std::arch::x86_64::_mm_setr_epi8;

use crate::ARTKey;
use crate::keys::{*};

#[derive(Clone)]
pub enum ARTNode<K: ARTKey, V: Clone> {
    Inner(Box<ARTInnerNode<K, V>>, ByteKey),
    Leaf(ARTLeaf<K, V>),
}

pub type ARTLink<K, V> = Option<Box<ARTNode<K, V>>>;

#[derive(Clone)]
pub enum ARTInnerNode<K: ARTKey, V: Clone> {
    Inner4(ARTInner4<K, V>),
    Inner16(ARTInner16<K, V>),
    Inner48(ARTInner48<K, V>),
    Inner256(ARTInner256<K, V>),
}

#[derive(Clone)]
pub struct ARTInner4<K: ARTKey, V: Clone> {
    pkey_size: u8,
    keys: [Option<u8>; 4],
    children: [ARTLink<K, V>; 4],
    children_num: u8,
}

use std::arch::x86_64::__m128i;

#[derive(Clone)]
pub struct ARTInner16<K: ARTKey, V: Clone> {
    pkey_size: u8,
    keys: __m128i,
    children: [ARTLink<K, V>; 16],
    children_num: u8,
}

#[derive(Clone)]
pub struct ARTInner48<K: ARTKey, V: Clone> {
    pkey_size: u8,
    keys: [Option<u8>; 256],
    children: [ARTLink<K, V>; 48],
    children_num: u8,
}

#[derive(Clone)]
pub struct ARTInner256<K: ARTKey, V: Clone> {
    pkey_size: u8,
    children: [ARTLink<K, V>; 256],
    children_num: u8,
}

#[derive(Clone)]
pub struct ARTLeaf<K: ARTKey, V: Clone>{
    key: ByteKey,
    value: V,
    _marker: PhantomData<K>,
}

impl<K: ARTKey, V: Clone> ARTLeaf<K, V> {
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

macro_rules! initialize_array {
    ($l: tt, $t: ty) => {{
        let mut arr: [MaybeUninit<$t>; $l] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        for item in &mut arr[..] {
            item.write(None);
        }

        unsafe { mem::transmute::<_, [$t; $l]>(arr) }
    }};
}

impl<'a, K: ARTKey, V: Clone> ARTInnerNode<K, V> {
    pub fn new_inner_4(pkey_size: u8) -> Box<ARTInnerNode<K, V>> {
        Box::new(ARTInnerNode::Inner4(ARTInner4 {
            keys: Default::default(),
            children: Default::default(),
            pkey_size,
            children_num: 0
        }))
    }


    // pub fn new_inner_48(pkey_size: u8) -> Box<ARTInnerNode<K, V>> {
    //     let children = {
    //         let mut arr: [MaybeUninit<Option<Box<ARTNode<K, V>>>>; 48] = unsafe {
    //             MaybeUninit::uninit().assume_init()
    //         };

    //         for item in &mut arr[..] {
    //             item.write(None);
    //         }

    //         unsafe { mem::transmute::<_, [Option<Box<ARTNode<K, V>>>; 48]>(arr) }
    //     };

    //     Box::new(ARTInnerNode::Inner48(ARTInner48 {
    //         keys: Default::default(),
    //         children,
    //         children_num: 0,
    //         pkey_size,
    //     }))
    // }

    // pub fn new_inner_256(pkey_size: u8) -> Box<ARTInnerNode<K, V>> {
    //     let children = {
    //         let mut arr: [MaybeUninit<Option<Box<ARTNode<K, V>>>>; 256] = unsafe {
    //             MaybeUninit::uninit().assume_init()
    //         };

    //         for item in &mut arr[..] {
    //             item.write(None);
    //         }

    //         unsafe { mem::transmute::<_, [Option<Box<ARTNode<K, V>>>; 256]>(arr) }
    //     };

    //     Box::new(ARTInnerNode::Inner256(ARTInner256 {
    //         children,
    //         children_num: 0,
    //         pkey_size,
    //     }))
    // }

    pub fn partial_key_size(&self) -> u8 {
        match self {
            ARTInnerNode::Inner4(node) => node.pkey_size,
            ARTInnerNode::Inner16(node) => node.pkey_size,
            ARTInnerNode::Inner48(node) => node.pkey_size,
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
            }
            ARTInnerNode::Inner16(node) => {
                unsafe {
                    let shifted = _mm_slli_si128::<1>(node.keys);
                    let new_key = _mm_setr_epi8(key_byte as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
                    node.keys = _mm_xor_si128(shifted, new_key);
                }
            }
            ARTInnerNode::Inner48(node) => {
                node.children[node.children_num as usize] = Some(new_node);
                node.keys[key_byte as usize] = Some(node.children_num);
                node.children_num += 1;
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
            ARTInnerNode::Inner16(node) => {
                let key = unsafe { _mm_set1_epi8(key_byte as i8) };
                let cmp = unsafe { _mm_cmpeq_epi8(key, node.keys) };
                let mask = (1 << node.children_num) - 1;
                let bitfield = unsafe { _mm_movemask_epi8(cmp) & mask };

                if bitfield > 0 {
                    Some(&mut node.children[bitfield.trailing_zeros() as usize] as *mut ARTLink<K, V>)
                } else {
                    None
                }
            }
            ARTInnerNode::Inner48(node) => {
                let index = node.keys[key_byte as usize];
                index.map(|i| &mut node.children[i as usize] as *mut ARTLink<K, V>)
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
            ARTInnerNode::Inner16(node) => {
                let key = unsafe { _mm_set1_epi8(key_byte as i8) };
                let cmp = unsafe { _mm_cmpeq_epi8(key, node.keys) };
                let mask = (1 << node.children_num) - 1;
                let bitfield = unsafe { _mm_movemask_epi8(cmp) & mask };

                if bitfield > 0 {
                    node.children[bitfield.trailing_zeros() as usize].as_ref()
                } else {
                    None
                }
            }
            ARTInnerNode::Inner48(node) => {
                let index = node.keys[key_byte as usize];
                index.map(|i| node.children[i as usize].as_ref()).flatten()
            }
            ARTInnerNode::Inner256(node) => node.children[key_byte as usize].as_ref(),
        }
    }

    
    pub fn grow(self) -> Box<ARTInnerNode<K, V>> {
        Box::new(
            match self {
                ARTInnerNode::Inner4(mut inner_node) => {
                    assert!(inner_node.children_num == 4);

                    let mut children: [ARTLink<K, V>; 16] = Default::default();

                    for (i, child) in inner_node.children.iter_mut().enumerate() {
                        children[i] = child.take();
                    }

                    let keys = unsafe {
                        _mm_setr_epi8(inner_node.keys[0].unwrap() as i8,
                                      inner_node.keys[1].unwrap() as i8,
                                      inner_node.keys[2].unwrap() as i8,
                                      inner_node.keys[3].unwrap() as i8,
                                      0,0,0,0,0,0,0,0,0,0,0,0)
                    };

                    ARTInnerNode::Inner16(ARTInner16 {
                        children_num: inner_node.children_num,
                        pkey_size: inner_node.pkey_size,
                        keys,
                        children
                    })
                }
                ARTInnerNode::Inner16(mut inner_node) => {
                    assert!(inner_node.children_num == 16);

                    let mut children: [ARTLink<K, V>; 48] = initialize_array!(48, ARTLink<K, V>);
                    let mut keys: [Option<u8>; 256] = initialize_array!(256, Option<u8>);

                    // children[..16].clone_from_slice(&inner_node.children);
                    unroll! {
                        for i in 0..16 {
                            children[i] = inner_node.children[i];
                            let byte = unsafe { _mm_extract_epi8(inner_node.keys, i as i32) };
                            keys[byte as usize] = Some(i as u8);
                        }
                    }
                    ARTInnerNode::Inner48(ARTInner48 {
                        children,
                        keys,
                        pkey_size: inner_node.pkey_size,
                        children_num: inner_node.children_num,
                    })
                }
                ARTInnerNode::Inner48(mut inner_node) => {
                    assert!(inner_node.children_num == 48);
                    let mut children = initialize_array!(256, ARTLink<K, V>);

                    for i in 0..inner_node.keys.len() {
                        if let Some(old_index) = inner_node.keys[i] {
                            children[i] = inner_node.children[old_index as usize].take();
                        }
                    }
                    ARTInnerNode::Inner256(ARTInner256 {
                        pkey_size: inner_node.pkey_size,
                        children,
                        children_num: inner_node.children_num,
                    })
                }
                ARTInnerNode::Inner256(_) => panic!("This node cannot grow!"),
            }
        )
    }
}

