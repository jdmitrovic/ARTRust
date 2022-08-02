use rust_art::ARTree;
use criterion::{ criterion_group, criterion_main, Criterion, BenchmarkId };

use rand_pcg::Pcg64;
use rand::{ SeedableRng, Rng };
use std::collections::{ BTreeMap, HashMap };

const SEED: u64 = 59;

fn art_insert(art: &mut ARTree<u64, u64>, keys: &Vec<u64>) {
    for key in keys.iter() {
        art.insert(*key, *key + 1);
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

fn art_get(art: &mut ARTree<u64, u64>, keys: &Vec<u64>) {
    for key in keys.iter() {
        assert_eq!(*key + 1, *art.get(*key).unwrap());
    }
}

fn hmap_get(hmap: &mut HashMap<u64, u64>, keys: &Vec<u64> ) {
    for key in keys.iter() {
        assert_eq!(*key + 1, *hmap.get(key).unwrap());
    }
}

fn btree_get(btree: &mut BTreeMap<u64, u64>, keys: &Vec<u64>) {
    for key in keys.iter() {
        assert_eq!(*key + 1, *btree.get(key).unwrap());
    }
}

fn bench_inserts(c: &mut Criterion) {
    let mut rng = Pcg64::seed_from_u64(SEED);

    let mut group = c.benchmark_group("Inserts");
    for i in (200_000..4_000_001).step_by(200_000) {
        let mut keys: Vec<u64> = vec![0; i];
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

fn bench_gets(c: &mut Criterion) {
    let mut rng = Pcg64::seed_from_u64(SEED);

    let mut group = c.benchmark_group("Gets");
    for i in (200_000..4_000_001).step_by(200_000) {
        let mut keys: Vec<u64> = vec![0; i];
        rng.fill(&mut keys[..]);
        let mut art = ARTree::<u64, u64>::new();
        let mut hmap = HashMap::<u64, u64>::new();
        let mut btree = BTreeMap::<u64, u64>::new();

        for key in keys.iter() {
            art.insert(*key, *key + 1);
            hmap.insert(*key, *key + 1);
            btree.insert(*key, *key + 1);
        }

        group.bench_with_input(BenchmarkId::new("ART", i), &keys,
            |b, k| b.iter(|| art_get(&mut art, k)));
        group.bench_with_input(BenchmarkId::new("HashMap", i), &keys,
            |b, k| b.iter(|| hmap_get(&mut hmap, k)));
        group.bench_with_input(BenchmarkId::new("BTree", i), &keys,
            |b, k| b.iter(|| btree_get(&mut btree, k)));
    }
    group.finish();
}

criterion_group!(benches, bench_inserts, bench_gets);
criterion_main!(benches);
