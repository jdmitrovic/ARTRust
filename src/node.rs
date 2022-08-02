use crate::keys::ByteKey;
use std::iter::zip;
use std::ptr::addr_of_mut;
use std::simd::u8x16;
use std::simd::Simd;

use crunchy::{self, unroll};

use auto_impl::auto_impl;
use enum_dispatch::enum_dispatch;

pub enum ARTNode<V> {
    Inner(ARTInnerNode<V>, ByteKey, Option<V>),
    Leaf(ARTLeaf<V>),
}

pub type ARTLink<V> = Option<ARTNode<V>>;

pub struct ARTInner4<V> {
    keys: [Option<u8>; 4],
    children: [ARTLink<V>; 4],
    children_num: u8,
}

pub struct ARTInner16<V> {
    keys: u8x16,
    children: [ARTLink<V>; 16],
    children_num: u8,
}

pub struct ARTInner48<V> {
    keys: [Option<u8>; 256],
    children: [ARTLink<V>; 48],
    children_num: u8,
}

pub struct ARTInner256<V> {
    children: [ARTLink<V>; 256],
    children_num: u16,
}

pub struct ARTLeaf<V> {
    pkey: ByteKey,
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
    fn child_index(&self, key_byte: u8) -> Option<usize> {
        unroll! {
            for i in 0..4 {
                if self.keys[i as usize] == Some(key_byte) {
                    return Some(i as usize);
                }
            }
        }

        None
    }

    fn boxed() -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).children_num).write(0);
            unroll! {
                for i in 0..4 {
                    addr_of_mut!((*this).keys[i]).write(None);
                    addr_of_mut!((*this).children[i]).write(None);
                }
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner16<V> {
    fn child_index(&self, key_byte: u8) -> Option<usize> {
        assert!(self.children_num <= 16);

        let key: Simd<u8, 16> = u8x16::splat(key_byte);
        let cmp = self.keys.lanes_eq(key);

        for i in 0..self.children_num as usize {
            if cmp.test(i) {
                return Some(i);
            }
        }

        None
    }

    fn boxed() -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).children_num).write(0);
            addr_of_mut!((*this).keys).write([0; 16].into());
            unroll! {
                for i in 0..16 {
                    addr_of_mut!((*this).children[i]).write(None);
                }
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner48<V> {
    fn boxed() -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).children_num).write(0);
            for i in 0..256 {
                addr_of_mut!((*this).keys[i]).write(None);
            }

            unroll! {
                for i in 0..48 {
                    addr_of_mut!((*this).children[i]).write(None);
                }
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTInner256<V> {
    fn boxed() -> Box<Self> {
        let mut uninit = Box::<Self>::new_uninit();
        let this = uninit.as_mut_ptr();
        unsafe {
            addr_of_mut!((*this).children_num).write(0);
            for i in 0..256 {
                addr_of_mut!((*this).children[i]).write(None);
            }
            uninit.assume_init()
        }
    }
}

impl<V> ARTLeaf<V> {
    pub fn new(pkey: ByteKey, value: V) -> Self {
        ARTLeaf {
            pkey,
            value,
        }
    }

    pub fn pkey(&self) -> &[u8] {
        &self.pkey
    }

    pub fn pkey_mut(&mut self) -> &mut ByteKey {
        &mut self.pkey
    }

    pub fn take_pkey_and_value(self) -> (ByteKey, V) {
        (self.pkey, self.value)
    }

    pub fn shrink_pkey(&mut self, len: usize) {
        self.pkey.drain(0..self.pkey.len() - len);
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
    fn add_child(&mut self, pkey: ByteKey, value: V, key_byte: u8) {
        self.add_node(ARTNode::Leaf(ARTLeaf::new(pkey, value)), key_byte)
    }

    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8);

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>>;
    fn remove_child(&mut self, key_byte: u8) -> Option<V>;
    fn shrink(self) -> ARTInnerNode<V>;
    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>>;
    fn is_full(&self) -> bool;
    fn is_shrinkable(&self) -> bool;
    fn grow(self) -> ARTInnerNode<V>;
}

impl<V> InnerNode<V> for ARTInner4<V> {
    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());

        let num = self.children_num as usize;
        self.keys[num] = Some(key_byte);
        self.children[num] = Some(new_node);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        let i = self.child_index(key_byte)?;
        Some(&mut self.children[i] as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let index = self.child_index(key_byte)?;
        let end = self.children_num as usize - 1;

        if index != end {
            self.keys[index] = self.keys[end];
            self.children.swap(index, end);
        }

        self.keys[end] = None;
        self.children_num -= 1;
        self.children[end].take().unwrap().try_into_leaf_value()
    }

    fn shrink(self) -> ARTInnerNode<V> {
        panic!("This node cannot shrink!")
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let i = self.child_index(key_byte)?;
        self.children[i].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 4
    }

    fn is_shrinkable(&self) -> bool {
        false
    }

    fn grow(self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 4);

        let mut node = ARTInner16::boxed();
        node.children_num = self.children_num;

        for (new, old) in zip(&mut node.children, self.children) {
            *new = old;
        }

        unroll! {
            for i in 0..4 {
                node.keys[i] = self.keys[i].unwrap();
            }
        }

        node.into()
    }
}

impl<V> InnerNode<V> for ARTInner16<V> {
    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        assert!(!self.is_full());

        let num = self.children_num as usize;
        self.keys[num] = key_byte;
        self.children[num] = Some(new_node);
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
        self.children[end].take().unwrap().try_into_leaf_value()
    }

    fn shrink(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 4);

        let mut node = ARTInner4::boxed();

        unroll! {
            for i in 0..4 {
                node.children[i] = self.children[i].take();
                node.keys[i] = Some(self.keys[i]);
            }
        }

        node.children_num = self.children_num;
        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let index = self.child_index(key_byte)?;
        self.children[index].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 16
    }

    fn is_shrinkable(&self) -> bool {
        self.children_num <= 4
    }

    fn grow(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 16);

        let mut node = ARTInner48::boxed();
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
        let index = self.keys[key_byte as usize].take()?;
        let end = self.children_num - 1;

        if index != end {
            self.children.swap(index as usize, end as usize);
            self.keys.iter_mut().find(|x| **x == Some(end))
                                .unwrap()
                                .replace(index);
        }

        self.children_num -= 1;
        self.children[end as usize].take().unwrap().try_into_leaf_value()
    }

    fn shrink(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 16);

        let mut node = ARTInner16::boxed();
        let mut children_num: u8 = 0;

        for (i, index) in self.keys.into_iter().enumerate() {
            if let Some(idx) = index {
                node.children[children_num as usize] = self.children[idx as usize].take();
                node.keys[children_num as usize] = i as u8;
                children_num += 1;
            }
        }

        node.children_num = children_num;
        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        let i = self.keys[key_byte as usize]?;
        self.children[i as usize].as_ref()
    }

    fn is_full(&self) -> bool {
        self.children_num >= 48
    }

    fn is_shrinkable(&self) -> bool {
        self.children_num <= 16
    }

    fn grow(mut self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 48);

        let mut node = ARTInner256::boxed();

        for (i, &key) in self.keys.iter().enumerate() {
            if let Some(index) = key {
                node.children[i] = self.children[index as usize].take();
            }
        }

        node.children_num = self.children_num as u16;
        node.into()
    }
}

impl<V> InnerNode<V> for ARTInner256<V> {
    fn add_node(&mut self, new_node: ARTNode<V>, key_byte: u8) {
        self.children[key_byte as usize] = Some(new_node);
        self.children_num += 1;
    }

    fn find_child_mut(&mut self, key_byte: u8) -> Option<*mut ARTLink<V>> {
        let node = &mut self.children[key_byte as usize];

        if node.is_none() {
            return None;
        }

        Some(node as *mut _)
    }

    fn remove_child(&mut self, key_byte: u8) -> Option<V> {
        let child = self.children[key_byte as usize].take()?;
        self.children_num -= 1;
        child.try_into_leaf_value()
    }

    fn shrink(self) -> ARTInnerNode<V> {
        assert_eq!(self.children_num, 48);

        let mut node = ARTInner48::boxed();
        let mut children_num: u8 = 0;

        for (i, child) in self.children.into_iter().enumerate() {
            if child.is_some() {
                node.children[children_num as usize] = child;
                node.keys[i] = Some(children_num);
                children_num += 1;
            }
        }

        node.children_num = children_num;
        node.into()
    }

    fn find_child(&self, key_byte: u8) -> Option<&ARTNode<V>> {
        self.children[key_byte as usize].as_ref()
    }

    fn is_full(&self) -> bool {
        false
    }

    fn is_shrinkable(&self) -> bool {
        self.children_num <= 48
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
    pub fn new_inner_4() -> Self {
        let node = ARTInner4::boxed();
        Self::Inner4(node)
    }
}
