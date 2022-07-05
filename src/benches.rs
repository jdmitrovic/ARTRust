#[cfg(test)]
mod benches {
    use crate::ARTree;
    extern crate test;
    use test::Bencher;


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
    //     let mut keys: Vec<u64> = vec![0; 1000];
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

    #[bench]
    fn hmap_find_1k(b: &mut Bencher) {
        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 1000];
        rng.fill(&mut keys[..]);
        let mut hmap = HashMap::<u64, u64>::new();

        for key in &keys {
            hmap.insert(*key, *key + 1);
        }

        b.iter(|| {
            for key in &keys {
                assert_eq!(*hmap.get(key).unwrap(), *key + 1);
            }
        });
    }

    #[bench]
    fn btree_find_1k(b: &mut Bencher) {
        let mut rng = Pcg64::seed_from_u64(SEED);
        let mut keys: Vec<u64> = vec![0; 1000];
        let mut btree = BTreeMap::<u64, u64>::new();

        rng.fill(&mut keys[..]);

        for key in &keys {
            btree.insert(*key, *key + 1);
        }

        b.iter(|| {
            for key in &keys {
                assert_eq!(*btree.get(key).unwrap(), *key + 1);
            }
        });
    }
}
