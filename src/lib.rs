#![feature(box_patterns)]

pub mod node;
pub mod tree;
pub mod keys;

use node::ARTLink;
use std::marker::PhantomData;

pub struct ARTree<K: ARTKey, V> {
    root: ARTLink<V>,
    _marker: PhantomData<K>,
}

pub trait ARTKey {
    fn convert_to_bytes(self) -> Vec<u8>;
}

#[cfg(test)]
mod tests {
    use crate::ARTree;
    #[test]
    fn it_works() {
        // assert_eq!(2 + 2, 4);

        let mut art: ARTree<String, u32> = ARTree::new();

        art.insert(String::from("Jason"), 26);
        art.insert(String::from("Drake"), 21);
        art.insert(String::from("Nathaniel"), 54);
        art.insert(String::from("Velma"), 22);
        art.insert(String::from("Sabrina"), 55);
        art.insert(String::from("Mary"), 75);
        art.insert(String::from("Caleb"), 75);
        art.insert(String::from("Keith"), 75);
        art.insert(String::from("Linda"), 75);
        art.insert(String::from("Tina"), 75);
        art.insert(String::from("Emily"), 75);
        art.insert(String::from("Gordon"), 75);
        art.insert(String::from("Anna"), 75);
        art.insert(String::from("Haley"), 75);
        art.insert(String::from("Bruce"), 75);
        art.insert(String::from("Zane"), 75);
        art.insert(String::from("Wendell"), 33);
        art.insert(String::from("Rusty"), 44);
        art.insert(String::from("Jerry"), 23);
        art.insert(String::from("Jenny"), 23);
        art.insert(String::from("Jenson"), 23);
        art.insert(String::from("Jen"), 50);
        art.insert(String::from("Wendell"), 50);

        assert_eq!(26, *art.find(String::from("Jason")).unwrap());
        assert_eq!(21, *art.find(String::from("Drake")).unwrap());
        assert_eq!(54, *art.find(String::from("Nathaniel")).unwrap());
        assert_eq!(22, *art.find(String::from("Velma")).unwrap());
        assert_eq!(55, *art.find(String::from("Sabrina")).unwrap());
        assert_eq!(44, *art.find(String::from("Rusty")).unwrap());
        assert_eq!(23, *art.find(String::from("Jerry")).unwrap());
        assert_eq!(23, *art.find(String::from("Jenny")).unwrap());
        assert_eq!(23, *art.find(String::from("Jenson")).unwrap());
        assert_eq!(50, *art.find(String::from("Jen")).unwrap());
        assert_eq!(50, *art.find(String::from("Wendell")).unwrap());

        art.delete(String::from("Jenny"));
        art.delete(String::from("Jason"));
        art.delete(String::from("Jen"));
        art.delete(String::from("Caleb"));

        assert_eq!(None, art.find(String::from("Jenny")));
        assert_eq!(None, art.find(String::from("Jason")));
        assert_eq!(None, art.find(String::from("Jen")));
        assert_eq!(None, art.find(String::from("Caleb")));

        assert_eq!(21, *art.find(String::from("Drake")).unwrap());
        assert_eq!(54, *art.find(String::from("Nathaniel")).unwrap());
        assert_eq!(22, *art.find(String::from("Velma")).unwrap());
        assert_eq!(55, *art.find(String::from("Sabrina")).unwrap());
        assert_eq!(44, *art.find(String::from("Rusty")).unwrap());
        assert_eq!(23, *art.find(String::from("Jerry")).unwrap());
        assert_eq!(23, *art.find(String::from("Jenson")).unwrap());
        assert_eq!(50, *art.find(String::from("Wendell")).unwrap());
    }
}
