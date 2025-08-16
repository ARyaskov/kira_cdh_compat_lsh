# kira_cdh_compat_lsh

Candidate search (LSH - Locality-Sensitive Hashing) primitives for CD-HIT–style clustering pipelines.

This crate focuses on **fast candidate retrieval** between sequences represented as sets of pre-hashed k-mers (`u64`). It provides:

- **Signatures**: classical **MinHash** and **KMV** (bottom-k) sketches
- **LSH banding** to quickly retrieve near neighbors
- **Parallel build/query** (feature `parallel`, enabled by default)
- **Deterministic** mixing for reproducibility

> Out of scope: FASTA/FASTQ parsing and `.clstr` writing (handled by other crates in our stack).  
> Goal: plug this crate **between** your k-mer hasher and your greedy clusterer.

---

## Installation

```toml
[dependencies]
kira_cdh_compat_lsh = "*"

# Optional features (recommended):
# default = ["parallel"]
````

### Feature flags

* `parallel` *(default)* — enables `rayon` for parallel build/query
* `serde` — enables `serde`/`serde_bytes` for signature (de)serialization helpers in client code

Minimum supported Rust version (MSRV): **1.85**.

---

## Quick start

```rust
use kira_cdh_compat_lsh::{
    kmv::KmvSketch,
    lsh::{LshIndex, LshParams},
    sketch::jaccard_from_signatures,
};

// 1) You already have u64 k-mer hashes for each sequence:
let hashes_a: Vec<u64> = /* ... */ vec![11,12,13,100,101,102,1000,2000];
let hashes_b: Vec<u64> = /* ... */ vec![12,13,14,101,102,103,1000,3000];

// 2) Build compact signatures (KMV shown; MinHash also available):
let mut sa = KmvSketch::new(128);         // keep k=128 minima
for h in hashes_a { sa.update(h); }
let sig_a = sa.finish();                  // sorted ascending

let mut sb = KmvSketch::new(128);
for h in hashes_b { sb.update(h); }
let sig_b = sb.finish();

// 3) Build an LSH index and insert signatures
let params = LshParams::new(32, 4).unwrap(); // 32 bands × 4 rows = 128
let mut index = LshIndex::with_params(params);
index.insert(0, &sig_a).unwrap();
index.insert(1, &sig_b).unwrap();
index.build();

// 4) Query candidates for A
let cands = index.query_candidates(&sig_a, 1); // (id, collisions)
assert!(cands.iter().any(|(id, _)| *id == 1));

// 5) Optional: estimate Jaccard from two signatures
let j_est = jaccard_from_signatures(&sig_a, &sig_b);
eprintln!("Estimated Jaccard(A,B) ~ {j_est:.3}");
```

---

## Concepts

### Signatures

* **MinHash**: classical K-permutation min estimator. More arithmetic, best-known estimator for Jaccard.
  API: `minhash::MinHash::new(num_hashes, seed0).update(u64).finish() -> Vec<u64>`

* **KMV (bottom-k)**: keep the k smallest hashed values from a single hash stream; usually faster than MinHash and works very well as a candidate generator.
  API: `kmv::KmvSketch::new(k).update(u64).finish() -> Vec<u64> (sorted)`

> Both outputs are `Vec<u64>` signatures that feed directly into LSH banding.

### LSH banding

Split a signature (length `bands * rows`) into `bands` chunks of `rows` each.
Each chunk is folded into a deterministic 64-bit **band key** and placed in a bucket.
Two sequences are candidates if they share ≥ `min_collisions` band keys.

API:

```rust
let params = LshParams::new(bands, rows_per_band)?;
let mut index = LshIndex::with_params(params);
index.insert(seq_id, &signature)?;
index.build(); // optional finalize
let hits: Vec<(u32, u32)> = index.query_candidates(&signature, min_collisions);
```

---

## Choosing parameters

* **Signature length**: `bands × rows`. Typical values: **128–256** for high-identity clustering.
* **Heuristic for high identity (CD-HIT-like)**:

    * Prefer **more bands** (e.g., 32–64) with **small rows** (e.g., 4) to keep recall high.
    * Increase `min_collisions` to tighten precision for higher identity thresholds.
* **KMV vs MinHash**:

    * Use **KMV** for speed (single hash per k-mer).
    * Use **MinHash** if you rely on the classical per-position equality estimator.

> Mapping nucleotide/protein identity → Jaccard over k-mers depends on k and error model.
> Treat `min_collisions` as a **tunable gate** before the expensive, exact stage.

---

## Determinism & reproducibility

* Band keys use a fixed `splitmix64`-based fold (platform-independent).
* `LshIndex` uses `FxHasher` internally for fast deterministic maps.
* Signatures depend only on your input hash stream and chosen parameters.

---

## Parallelism

When `parallel` is enabled (default), building large indexes and batch querying can be parallelized in higher-level code 
(e.g., producing signatures in parallel and calling `insert` from worker threads). The internal data structures 
in this crate are optimized for **fast single-threaded insert/query**; batching at a higher level typically yields better scalability.

---

## Integration in a CD-HIT–style pipeline

```
FASTA/FASTQ ──► k-mer hashing (u64) ──► (KMV | MinHash) signature ──► LSH candidates
                                                       │
                                                       └─► Greedy clusterer (exact check, identity/coverage)
                                                                      └─► .clstr writer (CD-HIT-compatible)
```

This crate implements only the **bold** path: signature construction and candidate retrieval.
It intentionally does **not** depend on I/O or `.clstr` formats.

---

## API surface

* `kmv::KmvSketch` — KMV (bottom-k) signatures
* `minhash::MinHash` — classical MinHash signatures
* `lsh::{LshParams, LshIndex}` — banding & candidate retrieval
* `sketch::jaccard_from_signatures(a, b)` — simple similarity estimate on same-length signatures

---

## Benchmarks

Run locally:

```bash
cargo bench --all-features
```

Tips:

* Use **release** mode and pin CPU frequency if measuring.
* For realistic testing, generate signatures from your real k-mer streams.

---

## Limitations & roadmap

* Current buckets store `Vec<u32>` per band key. A compact arena layout (single `Vec<u32>` with offsets) can cut memory and speed up scans. Planned as a non-breaking internal change.
* No on-disk index format (keep this crate pure). Serialization of **signatures** can be handled in outer layers; an index serializer may be added later behind a feature.
* Heuristics for `min_collisions` are workload-dependent; expose your tuning knobs in the higher layer that knows k, alphabet, and identity threshold.

---

## Contributing

Contributions are welcome. Please run `cargo fmt`, `cargo clippy -D warnings`, and add tests for new behavior.

---

## License

GPLv2
