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
    fn new_inner4() -> ARTInnerNode<K, V> {
        ARTInnerNode::Inner4 {
            keys: [None; 4],
            children: [None; 4],
        }
    }

    fn insert(&mut self, key: K, value: V) -> &ARTLeaf<K, V> {
        let key_bytes : Vec<u8> = key.convert_into_bytes();

        match self {
            ARTInnerNode::Inner4 { keys, children } => {
                for k in keys {
                    let byte_check = k.get_or_insert();
                }
            }
            ARTInnerNode::Inner256 { children } => {},
        }
    }
}
