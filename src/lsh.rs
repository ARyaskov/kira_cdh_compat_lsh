//! LSH banding over MinHash/KMV signatures.
//!
//! The signature is a slice of u64 values. We split it into `bands` bands,
//! each of `rows_per_band` rows. Each band's chunk is folded to a single
//! 64-bit key and used to bucket sequence IDs.

use crate::errors::LshError;
use crate::util::hash_band;
use hashbrown::HashMap;
use rustc_hash::FxBuildHasher;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct LshParams {
    pub bands: usize,
    pub rows_per_band: usize,
}

impl LshParams {
    pub fn new(bands: usize, rows_per_band: usize) -> Result<Self, LshParamsError> {
        if bands == 0 || rows_per_band == 0 {
            return Err(LshParamsError::Zero);
        }
        Ok(Self {
            bands,
            rows_per_band,
        })
    }

    #[inline]
    pub fn signature_len(&self) -> usize {
        self.bands * self.rows_per_band
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LshParamsError {
    #[error("bands and rows_per_band must be non-zero")]
    Zero,
}

/// Read-only finalized index.
pub struct LshIndex {
    params: LshParams,
    // For each band, map band-key -> Vec<id>
    bands: Vec<HashMap<u64, Vec<u32>, FxBuildHasher>>,
    // Optional global store of signatures if you want to re-query without passing a signature.
    // We keep it off by default to avoid duplication; use the map below for convenience.
    #[allow(dead_code)]
    ids: Vec<u32>,
    #[allow(dead_code)]
    signatures: Vec<Arc<Vec<u64>>>,
    seed: u64,
}

impl LshIndex {
    pub fn with_params(params: LshParams) -> Self {
        let mut bands = Vec::with_capacity(params.bands);
        for _ in 0..params.bands {
            bands.push(HashMap::with_hasher(FxBuildHasher::default()));
        }
        Self {
            params,
            bands,
            ids: Vec::new(),
            signatures: Vec::new(),
            seed: 0xC0FFEEFADEu64, // deterministic default
        }
    }

    /// Insert a signature for `id`. Signature length must equal `bands*rows`.
    pub fn insert(&mut self, id: u32, signature: &[u64]) -> Result<(), LshError> {
        let need = self.params.signature_len();
        if signature.len() < need {
            return Err(LshError::ShortSignature {
                sig_len: signature.len(),
                need,
            });
        }
        // Insert into each band's bucket.
        for b in 0..self.params.bands {
            let start = b * self.params.rows_per_band;
            let end = start + self.params.rows_per_band;
            let key = hash_band(&signature[start..end], (b as u64) ^ self.seed);
            self.bands[b].entry(key).or_default().push(id);
        }
        Ok(())
    }

    /// Optional finalize step (reserved for future compaction).
    pub fn build(&mut self) {
        // Currently a no-op: data is already in-place.
        // Future: shrink_to_fit, sort buckets, convert to compact arenas, etc.
        for map in &mut self.bands {
            for (_k, v) in map.iter_mut() {
                v.shrink_to_fit();
            }
        }
    }

    /// Query candidates for a given signature. Returns (id, collisions) pairs.
    /// `min_collisions` is the minimum number of band collisions to report.
    pub fn query_candidates(&self, signature: &[u64], min_collisions: usize) -> Vec<(u32, u32)> {
        let need = self.params.signature_len();
        assert!(
            signature.len() >= need,
            "signature too short for LSH parameters"
        );

        // Local counter: id -> collision count
        let mut counts: HashMap<u32, u32, FxBuildHasher> =
            HashMap::with_hasher(FxBuildHasher::default());

        for b in 0..self.params.bands {
            let start = b * self.params.rows_per_band;
            let end = start + self.params.rows_per_band;
            let key = hash_band(&signature[start..end], (b as u64) ^ self.seed);
            if let Some(ids) = self.bands[b].get(&key) {
                for &id in ids {
                    *counts.entry(id).or_insert(0) += 1;
                }
            }
        }

        let mut out = Vec::with_capacity(counts.len());
        for (id, c) in counts {
            if (c as usize) >= min_collisions {
                out.push((id, c));
            }
        }
        // Optional: sort by decreasing collisions, then id for stability
        out.sort_unstable_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        out
    }
}
