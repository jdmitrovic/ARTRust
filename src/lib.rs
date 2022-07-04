#![feature(box_patterns)]
#![feature(test)]

pub mod node;
pub mod tree;
pub mod keys;

use node::ARTLink;
use keys::ARTKey;
use std::marker::PhantomData;

extern crate test;

pub struct ARTree<K: ARTKey, V> {
    root: ARTLink<V>,
    _marker: PhantomData<K>,
}

#[macro_export]
macro_rules! initialize_array {
   ($l: tt, $t: ty) => {{
        use std::mem::MaybeUninit;
        use std::mem::transmute;

        let mut arr: [MaybeUninit<$t>; $l] = unsafe {
            MaybeUninit::uninit().assume_init()
        };

        for item in &mut arr[..] {
            item.write(None);
        }

        unsafe { transmute::<_, [$t; $l]>(arr) }
   }};
}

#[cfg(test)]
mod tests {
    use crate::ARTree;
    use test::Bencher;

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

    use rand_pcg::Pcg64;
    use rand::{SeedableRng, Rng};
        use std::collections::{ BTreeMap, HashMap };

    const SEED: u64 = 59;

    #[bench]
    fn art_insert_1k(b: &mut Bencher) {
        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 1000];
        let mut art = ARTree::<u64, u64>::new();

        rng.fill(&mut keys[..]);
        b.iter(|| {
            for key in &keys {
                art.insert_or_update(*key, *key + 1);
            }
        });
    }

    #[bench]
    fn hashmap_insert_1k(b: &mut Bencher) {
        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 1000];
        let mut hmap = HashMap::<u64, u64>::new();

        rng.fill(&mut keys[..]);
        b.iter(|| {
            for key in &keys {
                hmap.insert(*key, *key + 1);
            }
        });
    }

    #[bench]
    fn btree_insert_1k(b: &mut Bencher) {
        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 1000];
        let mut btree = BTreeMap::<u64, u64>::new();

        rng.fill(&mut keys[..]);
        b.iter(|| {
            for key in &keys {
                btree.insert(*key, *key + 1);
            }
        });
    }

    // #[bench]
    // fn art_find_1k(b: &mut Bencher) {
    //     let mut rng = Pcg64::seed_from_u64(SEED);
    //     let mut keys: Vec<u64> = vec![0; 900];
    //     let mut art = ARTree::<u64, u64>::new();
    //     rng.fill(&mut keys[..]);

    //     for key in &keys {
    //         art.insert_or_update(*key, *key + 1);
    //     }

    //     b.iter(|| {
    //         for key in &keys {
    //             dbg!(*key);
    //             assert_eq!(*art.find(*key).unwrap(), *key + 1);
    //         }
    //     });
    // }

    // #[bench]
    // fn hmap_find_1k(b: &mut Bencher) {
    //     let mut rng = Pcg64::seed_from_u64(SEED);
    //     let mut keys: Vec<u64> = vec![0; 900];
    //     rng.fill(&mut keys[..]);
    //     let mut hmap = HashMap::<u64, u64>::new();

    //     for key in &keys {
    //         hmap.insert(*key, *key + 1);
    //     }

    //     b.iter(|| {
    //         for key in &keys {
    //             assert_eq!(*hmap.get(key).unwrap(), *key + 1);
    //         }
    //     });
    // }

    // #[bench]
    // fn btree_find_1k(b: &mut Bencher) {
    //     let mut rng = Pcg64::seed_from_u64(SEED);
    //     let mut keys: Vec<u64> = vec![0; 1000];
    //     let mut btree = BTreeMap::<u64, u64>::new();

    //     rng.fill(&mut keys[..]);

    //     for key in &keys {
    //         btree.insert(*key, *key + 1);
    //     }

    //     b.iter(|| {
    //         for key in &keys {
    //             assert_eq!(*btree.get(key).unwrap(), *key + 1);
    //         }
    //     });
    // }
}
