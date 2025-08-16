//! Classic MinHash with K seeded permutations via splitmix64.
//!
//! Use when you want strict MinHash semantics. For speed on huge inputs,
//! consider `kmv::KmvSketch`.

use crate::util::mix_with_seed;

pub struct MinHash {
    seeds: Vec<u64>,
    mins: Vec<u64>,
}

impl MinHash {
    /// Create a MinHash with `num_hashes = bands * rows_per_band`.
    pub fn new(num_hashes: usize, seed0: u64) -> Self {
        let mut seeds = Vec::with_capacity(num_hashes);
        // Derive per-hash seeds from seed0 using splitmix64 chain.
        let mut s = seed0;
        for _ in 0..num_hashes {
            s = crate::util::splitmix64(s);
            seeds.push(s);
        }
        Self {
            mins: vec![u64::MAX; num_hashes],
            seeds,
        }
    }

    /// Update with a pre-hashed k-mer value (u64).
    #[inline]
    pub fn update(&mut self, x: u64) {
        let n = self.seeds.len();
        // Tight loop: branchless update of minima across all seeds.
        for i in 0..n {
            let h = mix_with_seed(x, self.seeds[i]);
            // Min update
            if h < self.mins[i] {
                self.mins[i] = h;
            }
        }
    }

    /// Final signature (length = num_hashes).
    pub fn finish(self) -> Vec<u64> {
        self.mins
    }
}
