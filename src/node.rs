use std::rc::Rc;
use std::mem::MaybeUninit;
use std::mem::transmute;

use crunchy::{ self, unroll };

use std::arch::x86_64::_mm_set1_epi8;
use std::arch::x86_64::_mm_cmpeq_epi8;
use std::arch::x86_64::_mm_movemask_epi8;
use std::arch::x86_64::_mm_xor_si128;
use std::arch::x86_64::_mm_bslli_si128;
use std::arch::x86_64::_mm_bsrli_si128;
use std::arch::x86_64::_mm_andnot_si128;
use std::arch::x86_64::_mm_setr_epi8;
use std::arch::x86_64::_mm_extract_epi16;
use std::arch::x86_64::_mm_and_si128;
use std::arch::x86_64::_mm_or_si128;

use crate::keys::ByteKey;

pub enum ARTNode<V> {
    Inner(Box<ARTInnerNode<V>>, ByteKey, Option<V>),
    Leaf(ARTLeaf<V>),
}

pub type ARTLink<V> = Option<Box<ARTNode<V>>>;

pub enum ARTInnerNode<V> {
    Inner4(ARTInner4<V>),
    Inner16(ARTInner16<V>),
    Inner48(ARTInner48<V>),
    Inner256(ARTInner256<V>),
}

pub struct ARTInner4<V> {
    pkey_size: u8,
    keys: [Option<u8>; 4],
    children: [ARTLink<V>; 4],
    children_num: u8,
}

use std::arch::x86_64::__m128i;

pub struct ARTInner16<V> {
    pkey_size: u8,
    keys: __m128i,
    children: [ARTLink<V>; 16],
    children_num: u8,
}

pub struct ARTInner48<V> {
    pkey_size: u8,
    keys: [Option<u8>; 256],
    children: [ARTLink<V>; 48],
    children_num: u8,
}

pub struct ARTInner256<V> {
    pkey_size: u8,
    children: [ARTLink<V>; 256],
    children_num: u8,
}

pub struct ARTLeaf<V>{
    key: ByteKey,
    value: V,
}

impl<V> ARTLeaf<V> {
    pub fn new(byte_key: &ByteKey, value: V) -> Self {
        ARTLeaf {
            key: Rc::clone(byte_key),
            value,
        }
    }

    pub fn new_bytekey(key: &ByteKey, value: V) -> Self {
        ARTLeaf {
            key: Rc::clone(key),
            value,
        }
    }

    pub fn key(&self) -> &[u8] {
        &self.key
    }

    pub fn value(&self) -> &V {
        &self.value
    }

    pub fn take_value(self) -> V {
        self.value
    }

    pub fn change_value(&mut self, val: V) -> V {
        std::mem::replace(&mut self.value, val)
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

        unsafe { transmute::<_, [$t; $l]>(arr) }
    }};
}

impl<V> ARTInnerNode<V> {
    pub fn new_inner_4(pkey_size: u8) -> Box<ARTInnerNode<V>> {
        Box::new(ARTInnerNode::Inner4(ARTInner4 {
            keys: Default::default(),
            children: Default::default(),
            pkey_size,
            children_num: 0
        }))
    }

    pub fn partial_key_size(&self) -> u8 {
        match self {
            ARTInnerNode::Inner4(node) => node.pkey_size,
            ARTInnerNode::Inner16(node) => node.pkey_size,
            ARTInnerNode::Inner48(node) => node.pkey_size,
            ARTInnerNode::Inner256(node) => node.pkey_size,
        }
    }

    pub fn reduce_pkey_size(&mut self, r: u8) {
        match self {
            ARTInnerNode::Inner4(node) => node.pkey_size -= r,
            ARTInnerNode::Inner16(node) => node.pkey_size -= r,
            ARTInnerNode::Inner48(node) => node.pkey_size -= r,
            ARTInnerNode::Inner256(node) => node.pkey_size -= r,
        }
    }

    pub fn add_child(&mut self, byte_key: &ByteKey, value: V, key_byte: u8) {
        self.add_node(Box::new(ARTNode::Leaf(ARTLeaf::new(byte_key, value))), key_byte)
    }

    pub fn add_node(&mut self, new_node: Box<ARTNode<V>>, key_byte: u8) {
        assert!(self.is_full() == false);

        match self {
            ARTInnerNode::Inner4(node) => {
                let num = node.children_num as usize;
                node.keys[num].replace(key_byte);
                node.children[num].replace(new_node);
                node.children_num += 1;
            }
            ARTInnerNode::Inner16(node) => {
                unsafe {
                    let mask = _mm_setr_epi8(key_byte as i8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
                    let new_key = _mm_bslli_si128::<1>(node.keys);
                    node.keys = _mm_xor_si128(mask, new_key);
                }

                let num = node.children_num as usize;
                node.children[num].replace(new_node);
                node.children_num += 1;
            }
            ARTInnerNode::Inner48(node) => {
                node.children[node.children_num as usize] = Some(new_node);
                node.keys[key_byte as usize] = Some(node.children_num);
                node.children_num += 1;
            }
            ARTInnerNode::Inner256(node) => {
                node.children[key_byte as usize].replace(new_node).unwrap();
                node.children_num += 1;
            }
        }
    }

    pub fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        match self {
            ARTInnerNode::Inner4(node) => {
                let pos = node.keys.iter().position(|k| {
                    match k {
                        None => false,
                        Some(x) => *x == key_byte,
                    }
                });

                pos.map(move |i| &mut node.children[i] as *mut ARTLink<V>)
            }
            ARTInnerNode::Inner16(node) => {
                let key = unsafe { _mm_set1_epi8(key_byte as i8) };
                let cmp = unsafe { _mm_cmpeq_epi8(key, node.keys) };
                let mask = (1 << node.children_num) - 1;
                let bitfield = unsafe { _mm_movemask_epi8(cmp) & mask };

                if bitfield > 0 {
                    let index = node.children_num as usize - bitfield.trailing_zeros() as usize - 1;
                    Some(&mut node.children[index] as *mut ARTLink<V>)


                } else {
                    None
                }
            }
            ARTInnerNode::Inner48(node) => {
                let index = node.keys[key_byte as usize];
                index.map(|i| &mut node.children[i as usize] as *mut ARTLink<V>)
            }
            ARTInnerNode::Inner256(node) => {
                Some(&mut node.children[key_byte as usize] as *mut ARTLink<V>)
            }
        }
    }

    pub fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let child = match self {
            ARTInnerNode::Inner4(node) => {
                let pos = node.keys.iter().position(|k| {
                    match k {
                        None => false,
                        Some(x) => *x == key_byte,
                    }
                });

                pos.map(move |i| {
                    node.keys[i].take();
                    node.children[i].take()
                }).flatten()
            }
            ARTInnerNode::Inner16(node) => {
                let key = unsafe { _mm_set1_epi8(key_byte as i8) };
                let cmp = unsafe { _mm_cmpeq_epi8(key, node.keys) };
                let mask = (1 << node.children_num) - 1;
                let bitfield = unsafe { _mm_movemask_epi8(cmp) & mask };

                if bitfield > 0 {
                    let btz = bitfield.trailing_zeros() as usize;
                    let index = node.children_num as usize - btz - 1;

                    if btz != 0 {
                        unsafe {
                            //removing the key
                            let new_keys = _mm_andnot_si128(cmp, node.keys);
                            let replacement_key  = _mm_extract_epi16::<0>(node.keys).to_be_bytes()[0];
                            let new_mask = _mm_set1_epi8(replacement_key as i8);
                            let new_mask = _mm_and_si128(new_mask, cmp);
                            node.keys = _mm_or_si128(new_keys, new_mask);
                        }

                        node.children.as_mut().swap(index, node.children_num as usize - 1);
                    }

                    node.keys = unsafe { _mm_bsrli_si128::<1>(node.keys) };
                    node.children_num -= 1;
                    node.children[node.children_num as usize].take()
                } else {
                    None
                }
            }
            ARTInnerNode::Inner48(node) => {
                let index = node.keys[key_byte as usize].take();
                index.map(|i| node.children[i as usize].take()).flatten()
            }
            ARTInnerNode::Inner256(node) => node.children[key_byte as usize].take(),
        };

        if let Some(box ARTNode::Leaf(leaf)) = child {
            Some(leaf.take_value())
        } else {
            None
        }
    }

    pub fn shrink(self) -> Box<ARTInnerNode<V>> {

        unimplemented!();
    }

    pub fn find_child(&self, key_byte: u8) -> Option<&Box<ARTNode<V>>> {
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
                    let index = node.children_num as usize - bitfield.trailing_zeros() as usize - 1;
                    node.children[index].as_ref()
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

    pub fn is_full(&self) -> bool {
        match self {
            ARTInnerNode::Inner4(ref node)  => node.children_num >= 4,
            ARTInnerNode::Inner16(ref node) => node.children_num >= 16,
            ARTInnerNode::Inner48(ref node) => node.children_num >= 48,
            ARTInnerNode::Inner256(_) => false,
        }
    }

    pub fn remove_one_child(&mut self) {
        match self {
            ARTInnerNode::Inner4(ref mut node)  => node.children_num -= 1,
            ARTInnerNode::Inner16(ref mut node) => node.children_num -= 1,
            ARTInnerNode::Inner48(ref mut node) => node.children_num -= 1,
            ARTInnerNode::Inner256(ref mut node) => node.children_num -= 1,
        }
    }
    
    pub fn grow(self) -> Box<ARTInnerNode<V>> {
        Box::new(
            match self {
                ARTInnerNode::Inner4(mut inner_node) => {
                    assert!(inner_node.children_num == 4);

                    let mut children: [ARTLink<V>; 16] = Default::default();

                    for (i, child) in inner_node.children.iter_mut().enumerate() {
                        children[i] = child.take();
                    }

                    let keys = unsafe {
                        _mm_setr_epi8(inner_node.keys[3].unwrap() as i8,
                                     inner_node.keys[2].unwrap() as i8,
                                     inner_node.keys[1].unwrap() as i8,
                                     inner_node.keys[0].unwrap() as i8,
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

                    let mut children: [ARTLink<V>; 48] = initialize_array!(48, ARTLink<V>);
                    let mut keys: [Option<u8>; 256] = initialize_array!(256, Option<u8>);

                    let bytes = unsafe {
                        transmute::<__m128i, [u8; 16]>(inner_node.keys)
                    };

                    unroll! {
                        for i in 0..16 {
                            children[i] = inner_node.children[i].take();
                            let byte = bytes[15 - i];
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
                    let mut children = initialize_array!(256, ARTLink<V>);

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

