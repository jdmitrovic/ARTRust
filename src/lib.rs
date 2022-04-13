#![feature(box_patterns)]

pub mod node;
pub mod tree;
pub mod keys;

use node::ARTLink;
use std::marker::PhantomData;

pub trait ARTKey {
    fn convert_to_bytes(self) -> Vec<u8>;
}

pub struct ARTree<K: ARTKey, V> {
    root: ARTLink<K, V>,
    _marker: PhantomData<K>,
}

#[cfg(test)]
mod tests {
    use crate::ARTree;
    #[test]
    fn it_works() {
        // assert_eq!(2 + 2, 4);

        let mut art: ARTree<String, u32> = ARTree::new();

        art.insert(String::from("Jovan"), 26);
        art.insert(String::from("Djordje"), 21);
        art.insert(String::from("Nenad"), 54);
        art.insert(String::from("Vesna"), 22);
        art.insert(String::from("Svetlana"), 55);
        art.insert(String::from("Gordana"), 75);

        assert_eq!(26, *art.find(String::from("Jovan")).unwrap());
        assert_eq!(21, *art.find(String::from("Djordje")).unwrap());
        assert_eq!(54, *art.find(String::from("Nenad")).unwrap());
        assert_eq!(22, *art.find(String::from("Vesna")).unwrap());
        assert_eq!(55, *art.find(String::from("Svetlana")).unwrap());
        assert_eq!(75, *art.find(String::from("Gordana")).unwrap());
    }
}
