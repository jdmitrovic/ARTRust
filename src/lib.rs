mod node;
mod tree;
mod keys;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

use node::ARTLink;
use std::marker::PhantomData;

pub trait ARTKey {
    fn convert_to_bytes(self) -> Vec<u8>;
}

pub struct ARTree<K: ARTKey, V> {
    root: ARTLink<V>,
    _marker: PhantomData<K>,
}
