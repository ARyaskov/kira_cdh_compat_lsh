use kira_cdh_compat_lsh::{
    kmv::KmvSketch,
    lsh::{LshIndex, LshParams},
    sketch::jaccard_from_signatures,
};

fn main() {
    // Toy hashed k-mers:
    let a = vec![11u64, 12, 13, 100, 101, 102, 1000, 2000];
    let b = vec![12u64, 13, 14, 101, 102, 103, 1000, 3000];

    // KMV sketch
    let k = 128;
    let mut sa = KmvSketch::new(k);
    for h in &a {
        sa.update(*h);
    }
    let sig_a = sa.finish();

    let mut sb = KmvSketch::new(k);
    for h in &b {
        sb.update(*h);
    }
    let sig_b = sb.finish();

    let params = LshParams::new(32, 4).unwrap(); // 32*4=128
    let mut index = LshIndex::with_params(params);
    index.insert(0, &sig_a).unwrap();
    index.insert(1, &sig_b).unwrap();
    index.build();

    let cands = index.query_candidates(&sig_a, 1);
    println!("Candidates for A: {:?}", cands);

    let j = jaccard_from_signatures(&sig_a, &sig_b);
    println!("Estimated Jaccard(A,B): {:.3}", j);
}
