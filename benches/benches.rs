use clap::ValueEnum;
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kvs::{Engine, Storage};
use once_cell::sync::Lazy;
use rand::{Rng, SeedableRng, distributions::Alphanumeric, rngs::SmallRng};
use tempfile::TempDir;

const NUM_VALS: usize = 10;
const KEY_SIZE_SEED: u64 = 233;
const KEY_SEED: u64 = 757;
const VALUE_SIZE_SEED: u64 = 2041;
const VALUE_SEED: u64 = 1024;
const READ_SEED: u64 = 999;

fn get_size(seed: u64) -> [usize; NUM_VALS] {
    let mut r: SmallRng = SeedableRng::seed_from_u64(seed);
    let mut res = [0; NUM_VALS];
    for val in res.iter_mut() {
        *val = r.gen_range(1, 100_000);
    }
    res
}

fn get_vals(seed: u64, size: &[usize]) -> Vec<String> {
    let mut r: SmallRng = SeedableRng::seed_from_u64(seed);
    let mut res = vec![];
    for s in size {
        let key = r.sample_iter(&Alphanumeric).take(*s).collect();
        res.push(key);
    }
    res
}

static KEY_SIZES: Lazy<[usize; NUM_VALS]> = Lazy::new(|| get_size(KEY_SIZE_SEED));
static VAL_SIZES: Lazy<[usize; NUM_VALS]> = Lazy::new(|| get_size(VALUE_SIZE_SEED));

static KEYS: Lazy<Vec<String>> = Lazy::new(|| get_vals(KEY_SEED, &*KEY_SIZES));
static VALS: Lazy<Vec<String>> = Lazy::new(|| get_vals(VALUE_SEED, &*VAL_SIZES));

fn bench_write(c: &mut Criterion) {
    for engine in Engine::value_variants() {
        let mut bench_write = c.benchmark_group("bench_write");
        let keys = &KEYS;
        let vals = &VALS;

        let engine_name = engine.to_string();
        let write_id = format!("{}-write", engine_name);
        bench_write.bench_function(write_id, |b| {
            b.iter_batched(
                || {
                    let tempdir = TempDir::new().unwrap().path().to_path_buf();
                    Storage::build(tempdir, *engine).unwrap()
                },
                |mut kv| {
                    for i in 0..NUM_VALS {
                        let key = keys[i].clone();
                        let val = vals[i].clone();
                        kv.set(key, val).unwrap();
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
}

fn bench_read(c: &mut Criterion) {
    for engine in Engine::value_variants() {
        let mut bench_read = c.benchmark_group("bench_read");

        let keys = &KEYS;
        let vals = &VALS;

        let engine_name = engine.to_string();
        let read_id = format!("{}-read", engine_name);
        bench_read.bench_function(read_id, |b| {
            b.iter_batched(
                || {
                    let tempdir = TempDir::new().unwrap().path().to_path_buf();
                    Storage::build(tempdir, *engine).unwrap()
                },
                |mut kv| {
                    let mut r: SmallRng = SeedableRng::seed_from_u64(READ_SEED);
                    // read 1000 times
                    for _ in 0..1000 {
                        let index = r.gen_range(0, NUM_VALS);
                        let key = keys[index].to_owned();
                        assert_eq!(Some(vals[index].clone()), kv.get(key).unwrap());
                    }
                },
                BatchSize::SmallInput,
            )
        });
    }
}

criterion_group!(engine, bench_write, bench_read);
criterion_main!(engine);
