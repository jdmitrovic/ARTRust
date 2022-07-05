use std::iter::zip;
use std::ptr::addr_of_mut;
use std::rc::Rc;
use std::simd::{u8x16, ToBitMask};

use crunchy::{self, unroll};

use auto_impl::auto_impl;
use enum_dispatch::enum_dispatch;

use crate::keys::ByteKey;

pub enum ARTNode<V> {
    Inner(ARTInnerNode<V>, ByteKey, Option<V>),
    Leaf(ARTLeaf<V>),
}

pub type ARTLink<V> = Option<ARTNode<V>>;

pub struct ARTInner4<V> {
    pkey_size: u8,
    keys: [Option<u8>; 4],
    children: [ARTLink<V>; 4],
    children_num: u8,
}

pub struct ARTInner16<V> {
    pkey_size: u8,
    keys: u8x16,
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

pub struct ARTLeaf<V> {
    key: ByteKey,
    value: V,
}

impl<V> ARTNode<V> {
    fn try_into_leaf_value(self) -> Option<V> {
        match self {
            ARTNode::Leaf(leaf) => Some(leaf.value),
            ARTNode::Inner(..) => None,
        }
    }
}

impl<V> ARTInner4<V> {
    fn key(&self, key_byte: u8) -> Option<usize> {
        self.keys.iter().position(|k| *k == Some(key_byte))
    }

    fn boxed(pkey_size: u8) -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).pkey_size).write(pkey_size);
            addr_of_mut!((*this).children_num).write(0);
            for i in 0..4 {
                addr_of_mut!((*this).keys[i]).write(None);
                addr_of_mut!((*this).children[i]).write(None);
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner16<V> {
    fn child_index(&self, key_byte: u8) -> Option<usize> {
        let key = u8x16::splat(key_byte);
        let cmp = self.keys.lanes_eq(key);
        if !cmp.any() {
            return None;
        }
        let mask = (1 << self.children_num) - 1; // TODO: probably wrong?
        let bitfield = cmp.to_bitmask() & mask;

        if bitfield != 0 {
            Some(bitfield.leading_zeros() as usize)
        } else {
            None
        }
    }

    fn boxed(pkey_size: u8) -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).pkey_size).write(pkey_size);
            addr_of_mut!((*this).children_num).write(0);
            addr_of_mut!((*this).keys).write([0; 16].into());
            for i in 0..16 {
                addr_of_mut!((*this).children[i]).write(None);
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner48<V> {
    fn boxed(pkey_size: u8) -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).pkey_size).write(pkey_size);
            addr_of_mut!((*this).children_num).write(0);
            for i in 0..256 {
                addr_of_mut!((*this).keys[i]).write(None);
            }
            for i in 0..48 {
                addr_of_mut!((*this).children[i]).write(None);
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner256<V> {
    fn boxed(pkey_size: u8) -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).pkey_size).write(pkey_size);
            addr_of_mut!((*this).children_num).write(0);
            for i in 0..256 {
                addr_of_mut!((*this).children[i]).write(None);
            }
            uninit.assume_init()
        }
    }
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

#[enum_dispatch]
#[auto_impl(Box)]
pub trait InnerNode<V> {
    fn partial_key_size(&self) -> u8;
    fn reduce_pkey_size(&mut self, r: u8);

    fn add_child(&mut self, byte_key: &ByteKey, value: V, key_byte: u8) {
        self.add_node(ARTNode::Leaf(ARTLeaf::new(byte_key, value)), key_byte)
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8);

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>>;
    fn remove_child(&mut self, key_byte: u8) -> Option<V>;
    fn shrink(self) -> ARTInnerNode<V>;
    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>>;
    fn is_full(&self) -> bool;
    fn remove_one_child(&mut self);
    fn grow(self) -> ARTInnerNode<V>;
}

impl<V> InnerNode<V> for ARTInner4<V> {
    fn partial_key_size(&self) -> u8 {
        self.pkey_size
    }

    fn reduce_pkey_size(&mut self, r: u8) {
        self.pkey_size -= r;
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());
        let num = self.children_num as usize;
        self.keys[num] = Some(key_byte);
        self.children[num] = Some(new_node);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        let i = self.key(key_byte)?;
        Some(&mut self.children[i] as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let i = self.key(key_byte)?;
        self.keys[i] = None;
        self.children[i].take()?.try_into_leaf_value()
    }

    fn shrink(self) -> ARTInnerNode<V> {
        panic!("This node cannot shrink!")
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let i = self.key(key_byte)?;
        self.children[i].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 4
    }

    fn remove_one_child(&mut self) {
        self.children_num -= 1;
    }

    fn grow(self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 4);

        let mut node = ARTInner16::boxed(self.pkey_size);
        node.children_num = self.children_num;

        for (new, old) in zip(&mut node.children, self.children) {
            *new = old;
        }

        for i in 0..4 {
            node.keys[i] = self.keys[i].unwrap();
        }

        node.into()
    }
}

impl<V> InnerNode<V> for ARTInner16<V> {
    fn partial_key_size(&self) -> u8 {
        self.pkey_size
    }

    fn reduce_pkey_size(&mut self, r: u8) {
        self.pkey_size -= r;
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());

        let num = self.children_num as usize;
        self.keys[num] = key_byte;
        self.children[num].replace(new_node);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        let index = self.child_index(key_byte)?;
        Some(&mut self.children[index] as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let index = self.child_index(key_byte)?;

        let end = self.children_num as usize - 1;
        if index != end {
            self.keys[index] = self.keys[end];
            self.children.swap(index, end);
        }

        self.children_num -= 1;
        self.children[end].take()?.try_into_leaf_value()
    }

    fn shrink(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 4);

        let mut node = ARTInner4::boxed(self.pkey_size);

        unroll! {
            for i in 0..4 {
                node.children[i] = self.children[i].take();
                node.keys[self.keys[i] as usize] = Some(i as u8);
            }
        }

        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let index = self.child_index(key_byte)?;
        self.children[index].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 16
    }

    fn remove_one_child(&mut self) {
        self.children_num -= 1;
    }

    fn grow(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 16);

        let mut node = ARTInner48::boxed(self.pkey_size);
        node.children_num = self.children_num;

        unroll! {
            for i in 0..16 {
                node.children[i] = self.children[i].take();
                node.keys[self.keys[i] as usize] = Some(i as u8);
            }
        }

        node.into()
    }
}

impl<V> InnerNode<V> for ARTInner48<V> {
    fn partial_key_size(&self) -> u8 {
        self.pkey_size
    }

    fn reduce_pkey_size(&mut self, r: u8) {
        self.pkey_size -= r;
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());

        self.children[self.children_num as usize] = Some(new_node);
        self.keys[key_byte as usize] = Some(self.children_num);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        let i = self.keys[key_byte as usize]?;
        Some(&mut self.children[i as usize] as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let i = self.keys[key_byte as usize].take()?;
        self.children[i as usize].take()?.try_into_leaf_value()
    }

    fn shrink(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 16);

        let mut node = ARTInner16::boxed(self.pkey_size);

        // temp_keys wasn't getting used
        for (i, index) in self.keys.into_iter().flatten().enumerate() {
            node.children[i] = self.children[index as usize].take();
            node.children_num += 1;
        }

        for i in 0..16 {
            node.keys[i] = self.keys[i].unwrap();
        }

        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let i = self.keys[key_byte as usize]?;
        self.children[i as usize].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 48
    }

    fn remove_one_child(&mut self) {
        self.children_num -= 1;
    }

    fn grow(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 48);

        let mut node = ARTInner256::boxed(self.pkey_size);
        node.children_num = self.children_num;

        for (child, key) in zip(&mut node.children, self.keys) {
            if let Some(old_index) = key {
                *child = self.children[old_index as usize].take();
            }
        }

        node.into()
    }
}

impl<V> InnerNode<V> for ARTInner256<V> {
    fn partial_key_size(&self) -> u8 {
        self.pkey_size
    }

    fn reduce_pkey_size(&mut self, r: u8) {
        self.pkey_size -= r;
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());

        self.children[key_byte as usize].replace(new_node);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        Some(&mut self.children[key_byte as usize] as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        self.children[key_byte as usize]
            .take()?
            .try_into_leaf_value()
    }

    fn shrink(self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 48);

        let mut node = ARTInner48::boxed(self.pkey_size);

        for (i, child) in self.children.into_iter().enumerate() {
            if child.is_some() {
                node.children[node.children_num as usize] = child;
                node.keys[i] = Some(node.children_num);
                node.children_num += 1;
            }
        }

        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        self.children[key_byte as usize].as_ref()
    }

    fn is_full(&self) -> bool {
        false
    }

    fn remove_one_child(&mut self) {
        self.children_num -= 1;
    }

    fn grow(self) -> ARTInnerNode<V> {
        panic!("This node cannot grow!")
    }
}

#[enum_dispatch(InnerNode<V>)]
pub enum ARTInnerNode<V> {
    Inner4(Box<ARTInner4<V>>),
    Inner16(Box<ARTInner16<V>>),
    Inner48(Box<ARTInner48<V>>),
    Inner256(Box<ARTInner256<V>>),
}

impl<V> ARTInnerNode<V> {
    pub fn new_inner_4(pkey_size: u8) -> Self {
        let (keys, children, children_num) = Default::default();
        Self::Inner4(Box::new(ARTInner4 {
            pkey_size,
            keys,
            children,
            children_num,
        }))
    }
}
