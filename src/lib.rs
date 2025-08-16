//! kira_cdh_compat_lsh
//!
//! Candidate search primitive for high-identity clustering pipelines
//! (e.g., CD-HIT-like). This crate provides:
//! - MinHash and KMV (bottom-k) sketches over pre-hashed k-mers (u64),
//! - Classic LSH banding to retrieve candidate neighbors,
//! - Parallel bulk build & queries (feature `parallel`).
//!
//! The crate **does not** parse FASTA/FASTQ and **does not** write `.clstr`;
//! it focuses solely on sketching and candidate retrieval.
//!
//! # Quick Start
//!
//! ```rust
//! use kira_cdh_compat_lsh::*;
//!
//! // Suppose you already have hashed k-mers for sequences:
//! let seq_a: Vec<u64> = vec![1, 2, 3, 10, 11, 12];
//! let seq_b: Vec<u64> = vec![2, 3, 4, 11, 12, 13];
//!
//! // Build a KMV sketch (fast single-hash approach):
//! let mut kmv = kmv::KmvSketch::new(128);
//! for h in &seq_a { kmv.update(*h); }
//! let sig_a = kmv.finish(); // Vec<u64> of length <= k (exactly k if enough items)
//!
//! let mut kmv2 = kmv::KmvSketch::new(128);
//! for h in &seq_b { kmv2.update(*h); }
//! let sig_b = kmv2.finish();
//!
//! // LSH parameters: 32 bands x 4 rows = 128
//! let params = lsh::LshParams::new(32, 4).unwrap();
//!
//! let mut index = lsh::LshIndex::with_params(params.clone());
//! index.insert(0, &sig_a);
//! index.insert(1, &sig_b);
//! index.build(); // finalize buckets (optional no-op for current implementation)
//!
//! // Query candidates for seq_a's signature:
//! let cands = index.query_candidates(&sig_a, 1); // min 1 band collision
//! // cands is Vec<(id, collisions)>
//! assert!(cands.iter().any(|(id, _)| *id == 1));
//!
//! // Jaccard estimate (KMV or MinHash signatures):
//! let j_est = sketch::jaccard_from_signatures(&sig_a, &sig_b);
//! eprintln!("Estimated Jaccard: {:.3}", j_est);
//! ```
//!
//! # Notes
//! - For MinHash, use `minhash::MinHash` with `num_hashes = bands * rows`.
//! - For KMV, you might prefer slightly larger k to reach stable estimates.
//! - LSH banding is deterministic and uses splitmix64 to map bands to buckets.

pub mod errors;
pub mod kmv;
pub mod lsh;
pub mod minhash;
pub mod sketch;
pub mod util;

pub use lsh::{LshIndex, LshParams};
