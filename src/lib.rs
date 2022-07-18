#![deny(rust_2018_idioms)]
#![feature(new_uninit, portable_simd)]

pub mod keys;
pub mod node;
pub mod tree;

use keys::ARTKey;
use node::ARTLink;
use std::marker::PhantomData;

pub struct ARTree<K: ARTKey, V> {
    root: ARTLink<V>,
    _marker: PhantomData<K>,
}

#[cfg(test)]
mod tests {
    use crate::ARTree;
    use rand_pcg::Pcg64;
    use rand::{ SeedableRng, Rng };

    #[test]
    fn it_works() {
        let mut art: ARTree<String, u32> = ARTree::new();

        art.insert_or_update(String::from("Jason"), 26);
        art.insert_or_update(String::from("Drake"), 21);
        art.insert_or_update(String::from("Nathaniel"), 54);
        art.insert_or_update(String::from("Velma"), 22);
        art.insert_or_update(String::from("Sabrina"), 55);
        art.insert_or_update(String::from("Mary"), 75);
        art.insert_or_update(String::from("Caleb"), 75);
        art.insert_or_update(String::from("Keith"), 75);
        art.insert_or_update(String::from("Linda"), 75);
        art.insert_or_update(String::from("Tina"), 75);
        art.insert_or_update(String::from("Emily"), 75);
        art.insert_or_update(String::from("Gordon"), 75);
        art.insert_or_update(String::from("Anna"), 75);
        art.insert_or_update(String::from("Haley"), 75);
        art.insert_or_update(String::from("Bruce"), 75);
        art.insert_or_update(String::from("Zane"), 75);
        art.insert_or_update(String::from("Wendell"), 33);
        art.insert_or_update(String::from("Rusty"), 44);
        art.insert_or_update(String::from("Jerry"), 23);
        art.insert_or_update(String::from("Jenny"), 23);
        art.insert_or_update(String::from("Jenson"), 23);
        art.insert_or_update(String::from("Jen"), 50);
        art.insert_or_update(String::from("Wendell"), 50);

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
        art.delete(String::from("Drake"));

        assert_eq!(None, art.find(String::from("Drake")));
        assert_eq!(None, art.find(String::from("Jenny")));
        assert_eq!(None, art.find(String::from("Jason")));
        assert_eq!(None, art.find(String::from("Jen")));
        assert_eq!(None, art.find(String::from("Caleb")));

        assert_eq!(54, *art.find(String::from("Nathaniel")).unwrap());
        assert_eq!(22, *art.find(String::from("Velma")).unwrap());
        assert_eq!(55, *art.find(String::from("Sabrina")).unwrap());
        assert_eq!(44, *art.find(String::from("Rusty")).unwrap());
        assert_eq!(23, *art.find(String::from("Jerry")).unwrap());
        assert_eq!(23, *art.find(String::from("Jenson")).unwrap());
        assert_eq!(50, *art.find(String::from("Wendell")).unwrap());
    }

    #[test]
    fn insert_100k() {
        const SEED: u64 = 59;

        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 100_000];
        rng.fill(&mut keys[..]);
        let mut art = ARTree::<u64, u64>::new();

        for &key in keys.iter() {
                art.insert_or_update(key, key + 1);
        }
    }

    #[test]
    fn insert_update_delete_find() {
        const SEED: u64 = 10;

        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 100_000];
        rng.fill(&mut keys[..]);
        let mut art = ARTree::<u64, u64>::new();

        for &key in keys.iter() {
            art.insert_or_update(key, key + 1);
        }

        for &key in keys.iter() {
            assert_eq!(key + 1, *art.find(key).unwrap());
        }

        const SEED_DEL: u64 = 13;

        for &key in keys.iter() {
            if key % SEED_DEL == 0 {
                art.delete(key);
            }
        }

        for &key in keys.iter() {
            if key % SEED_DEL == 0 {
                assert_eq!(None, art.find(key));
            } else {
                assert_eq!(key + 1, *art.find(key).unwrap());
            }
        }
    }
}
