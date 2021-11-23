use crate::ARTKey;

pub enum ARTNode<K: ARTKey, V> {
    Inner(ARTInnerNode<K, V>),
    Leaf(Box<ARTLeaf<K, V>>),
}

pub type ARTLink<K, V> = Option<ARTNode<K, V>>;

pub enum ARTInnerNode<K: ARTKey, V> {
    Inner4(ARTInner4),
    Inner256(ARTInner256),
}

pub struct ARTInner4<K, V> {
    partial_key: Vec<u8>,
    keys: [Option<u8>; 4],
    children: [ARTLink<K, V>; 4],
}

pub struct ARTInner256<K, V> {
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
}

impl<K: ARTKey, V> ARTInnerNode<K, V> {
    fn new_inner_4() -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner4(ARTInner4 {
            keys: [None; 4],
            children: [None; 4],
            partial_key: vec![],
        })
    }

    fn new_inner_256() -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner256(ARTInner256 {
            children: [None; 256],
            partial_key: vec![],
        })
    }

    pub fn add_child(&mut self, key: K, value: V, key_byte: u8) -> &ARTLeaf<K, V> {
        self.add_node(ARTNode::Leaf(Box::new(ARTLeaf::new(key, value))), key_byte)
    }

    pub fn add_node(&mut self, new_node: &ARTNode<K, V>, key_byte: u8) -> &ARTNode<K, V> {
        match self {
            ARTInnerNode::Inner4(node) => {
                for (i, key) in node.keys.iter().enumerate() {
                    if key != None { continue; }

                    key.insert(key_byte);
                    node.children[i].insert(new_node);
                    &node.children[i]
                }

               // node.grow()
            }
            ARTInnerNode::Inner256(node) => {
                node.children[key_byte].insert(new_node);
                &node.children[key_byte]
            },
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

pub struct Iter<'a, K, V> {
    next: Option<&'a ARTInnerNode<K, V>>,
    key_bytes: Vec<u8>,
    depth: u32,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = &'a ARTInnerNode<K, V>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.map(|inner_node| {
            match inner_node {
                ARTInnerNode::Inner4(node) => {
                    let pk = &self.key_bytes[self.depth..node.partial_key.len()];
                    if node.partial_key.iter().zip(pk).all(|a, b| a == b) == false {
                        self.next = None;
                        None
                    }

                    self.depth += node.partial_key.len();
                    let key_byte = self.key_bytes[self.depth];
                    let i = node.keys.iter().position(|k| k == key_byte);

                    if let Some(pos) = i {
                        self.depth += 1;

                    } else {
                        self.next = None;
                    }

                    &node
                }
                ARTInnerNode::Inner256(node) => {
                    let pk = &self.key_bytes[self.depth..node.partial_key.len()];
                    if node.partial_key.iter().zip(pk).all(|a, b| a == b) == false {
                        self.next = None;
                        None
                    }

                    let key_byte: u8 = self.key_bytes[self.depth];

                    self.depth += node.partial_key.len() + 1;
                    match node.children[key_byte] {
                        None => self.next = None,
                        Some(node) => {
                            match node {
                                ARTNode::Leaf(_) => None,
                                ARTNode::Inner(inner_node) => self.next = inner_node,
                            }
                        }
                    }
                }
            }
        })
    }
}
