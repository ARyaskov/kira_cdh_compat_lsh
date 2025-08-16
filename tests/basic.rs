use kira_cdh_compat_lsh::{
    kmv::KmvSketch,
    lsh::{LshIndex, LshParams},
    minhash::MinHash,
    sketch::jaccard_from_signatures,
};

#[test]
fn minhash_basic() {
    let mut mh = MinHash::new(64, 12345);
    for x in [1u64, 2, 3, 100, 101, 102] {
        mh.update(x);
    }
    let sig = mh.finish();
    assert_eq!(sig.len(), 64);
}

#[test]
fn kmv_basic() {
    let mut k = KmvSketch::new(64);
    for x in [1u64, 2, 3, 100, 101, 102] {
        k.update(x);
    }
    let sig = k.finish();
    assert!(sig.len() <= 64);
    assert!(sig.windows(2).all(|w| w[0] <= w[1]));
}

#[test]
fn lsh_query() {
    let params = LshParams::new(32, 4).unwrap();
    let mut idx = LshIndex::with_params(params);

    // Make two similar signatures
    let s1 = vec![1u64; 128];
    let mut s2 = s1.clone();
    s2[0] = 2;

    idx.insert(0, &s1).unwrap();
    idx.insert(1, &s2).unwrap();
    idx.build();

    let cands = idx.query_candidates(&s1, 1);
    assert!(cands.iter().any(|(id, _)| *id == 1));
}

#[test]
fn jaccard_estimate() {
    let a = vec![1u64; 128];
    let mut b = a.clone();
    b[64] = 42;
    let j = jaccard_from_signatures(&a, &b);
    assert!(j > 0.9);
}
