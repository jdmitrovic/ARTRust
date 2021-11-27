use std::mem::{ self, MaybeUninit };

use crate::ARTKey;

pub enum ARTNode<K: ARTKey, V> {
    Inner(ARTInnerNode<K, V>),
    Leaf(Box<ARTLeaf<K, V>>),
}

pub type ARTLink<K, V> = Option<ARTNode<K, V>>;

pub enum ARTInnerNode<K: ARTKey, V> {
    Inner4(Box<ARTInner4<K, V>>),
    Inner256(Box<ARTInner256<K, V>>),
}

pub struct ARTInner4<K: ARTKey, V> {
    partial_key: Vec<u8>,
    keys: [Option<u8>; 4],
    children: [ARTLink<K, V>; 4],
}

pub struct ARTInner256<K: ARTKey, V> {
    partial_key: Vec<u8>,
    children: [ARTLink<K, V>; 256],
}

pub struct ARTLeaf<K: ARTKey, V>{
    key: K,
    value: V,
}

impl<K: ARTKey, V> ARTLeaf<K, V> {
    pub fn new(key: K, value: V) -> Self {
        ARTLeaf {
            key,
            value,
        }
    }

    pub fn key(&self) -> &K {
        &self.key
    }
}

impl<K: ARTKey, V> ARTInnerNode<K, V> {
    pub fn new_inner_4() -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner4(Box::new(ARTInner4 {
            keys: Default::default(),
            children: Default::default(),
            partial_key: vec![],
        }))
    }

    pub fn new_inner_256() -> ARTInnerNode<K, V> {
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
            partial_key: vec![],
            children
        }))
    }

    fn partial_key(&self) -> &Vec<u8> {
        match self {
            ARTInnerNode::Inner4(node) => node.partial_key.as_ref(),
            ARTInnerNode::Inner256(node) => node.partial_key.as_ref()
        }
    }

    pub fn add_child(&mut self, key: K, value: V, key_byte: u8) {
        self.add_node(ARTNode::Leaf(Box::new(ARTLeaf::new(key, value))), key_byte)
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

               node.grow();
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
}

impl<K: ARTKey, V> ARTInner4<K, V> {
    fn grow(&mut self) -> ARTNode<K, V> {
        ARTNode::Inner(ARTInnerNode::new_inner_256())
    } 
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
                    self.depth += node.partial_key.len() as u32;

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

            if let Some(ARTNode::Inner(ref pn_inner)) = potential_next {
                let pk = &self.key_bytes[self.depth as usize..pn_inner.partial_key().len()];
                if pn_inner.partial_key().iter().zip(pk).all(|(a, b)| a == b) == false {
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
