use crate::ARTKey;

pub type ARTLink<K, V> = Option<Box<ARTNode<K, V>>>;

pub enum ARTNode<K: ARTKey, V> {
    Inner(ARTInnerNode<K, V>, Vec<u8>),
    Leaf(ARTLeaf<K, V>),
}

pub enum ARTInnerNode<K: ARTKey, V> {
    Inner4 {
        keys: [Option<u8>; 4],
        children: [ARTLink<K, V>; 4],
    },
    // Inner16(),
    // Inner48(),
    Inner256 {
        children: [ARTLink<K, V>; 256],
    },
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
        ARTInnerNode::Inner4 {
            keys: [None; 4],
            children: [None; 4],
        }
    }

    fn new_inner_256() -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner256 {
            children: [None; 256],
        }
    }

    fn add_child(&mut self, key: K, value: V, key_byte: u8) -> &ARTLeaf<K, V> {
        self.add_node(Box::new(ARTNode::Leaf(ARTLeaf::new(key, value))), key_byte)
    }

    fn add_node(&mut self, new_node: ARTNode<K, V>, key_byte: u8) -> &ARTNode<K, V> {
        match self {
            ARTInnerNode::Inner4 { keys, children } => {
                for (key, child) in self.keys.zip(self.children) {
                    if key != None { continue; }

                    key.insert(key_byte);
                    child.insert(new_node)
                }

               // node.grow()
            }
            ARTInnerNode::Inner256 { children } => {
                children[key_byte].insert(new_node)
            },
        }
    }
}
