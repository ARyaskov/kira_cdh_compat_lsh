use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kira_cdh_compat_lsh::{
    kmv::KmvSketch,
    lsh::{LshIndex, LshParams},
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

fn bench_build_query(c: &mut Criterion) {
    let mut rng = StdRng::seed_from_u64(42);
    let n = 10_000;
    let k = 128;

    let signatures: Vec<Vec<u64>> = (0..n)
        .map(|_| {
            let mut s = KmvSketch::new(k);
            for _ in 0..(k * 4) {
                s.update(rng.r#gen::<u64>());
            }
            s.finish()
        })
        .collect();

    let params = LshParams::new(32, 4).unwrap();

    c.bench_function("build_index_10k", |b| {
        b.iter_batched(
            || signatures.clone(),
            |sigs| {
                let mut idx = LshIndex::with_params(params.clone());
                for (i, sig) in sigs.iter().enumerate() {
                    idx.insert(i as u32, sig).unwrap();
                }
                idx.build();
            },
            BatchSize::SmallInput,
        )
    });

    c.bench_function("query_1k", |b| {
        let mut idx = LshIndex::with_params(params.clone());
        for (i, sig) in signatures.iter().enumerate() {
            idx.insert(i as u32, sig).unwrap();
        }
        idx.build();

        let q = &signatures[0];
        b.iter(|| {
            let r = idx.query_candidates(q, 1);
            criterion::black_box(r);
        });
    });
}

criterion_group!(benches, bench_build_query);
criterion_main!(benches);
