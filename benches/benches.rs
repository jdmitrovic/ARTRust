use rust_art::ARTree;
use criterion::{ criterion_group, criterion_main, Criterion, BenchmarkId };

use rand_pcg::Pcg64;
use rand::{ SeedableRng, Rng };
use std::collections::{ BTreeMap, HashMap };

const SEED: u64 = 59;

fn art_insert(art: &mut ARTree<u64, u64>, keys: &Vec<u64>) {
    for key in keys.iter() {
        art.insert_or_update(*key, *key + 1);
    }
}

fn hmap_insert(hmap: &mut HashMap<u64, u64>, keys: &Vec<u64> ) {
    for key in keys.iter() {
        hmap.insert(*key, *key + 1);
    }
}

fn btree_insert(btree: &mut BTreeMap<u64, u64>, keys: &Vec<u64>) {
    for key in keys.iter() {
        btree.insert(*key, *key + 1);
    }
}

fn bench_inserts(c: &mut Criterion) {
    let mut rng = Pcg64::seed_from_u64(SEED);

    let mut group = c.benchmark_group("Inserts");
    for i in [1000u64, 10000u64].iter() {
        let mut keys: Vec<u64> = vec![0; *i as usize];
        rng.fill(&mut keys[..]);
        let mut art = ARTree::<u64, u64>::new();
        let mut hmap = HashMap::<u64, u64>::new();
        let mut btree = BTreeMap::<u64, u64>::new();

        group.bench_with_input(BenchmarkId::new("ART", i), &keys,
            |b, k| b.iter(|| art_insert(&mut art, k)));
        group.bench_with_input(BenchmarkId::new("HashMap", i), &keys,
            |b, k| b.iter(|| hmap_insert(&mut hmap, k)));
        group.bench_with_input(BenchmarkId::new("BTree", i), &keys,
            |b, k| b.iter(|| btree_insert(&mut btree, k)));
    }
    group.finish();
}

// fn bench_finds(c: &mut Criterion) {
//     let mut rng = Pcg64::seed_from_u64(SEED);

//     let mut group = c.benchmark_group("Inserts");
//     for i in [1000u64, 10000u64].iter() {
//         let mut keys: Vec<u64> = vec![0; *i as usize];
//         rng.fill(&mut keys[..]);
//         let mut art = ARTree::<u64, u64>::new();
//         let mut hmap = HashMap::<u64, u64>::new();
//         let mut btree = BTreeMap::<u64, u64>::new();

//         group.bench_with_input(BenchmarkId::new("ART", i), &keys,
//             |b, k| b.iter(|| art_insert(&mut art, k)));
//         group.bench_with_input(BenchmarkId::new("HashMap", i), &keys,
//             |b, k| b.iter(|| hmap_insert(&mut hmap, k)));
//         group.bench_with_input(BenchmarkId::new("BTree", i), &keys,
//             |b, k| b.iter(|| btree_insert(&mut btree, k)));
//     }
//     group.finish();
// }

criterion_group!(benches, bench_inserts);
criterion_main!(benches);
