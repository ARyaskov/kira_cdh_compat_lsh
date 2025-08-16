//! KMV (k-minimum values, aka bottom-k) sketch over a single hash stream.
//! Typically faster than classical MinHash as it uses a single hash per element.

use std::cmp::Reverse;
use std::collections::BinaryHeap;

pub struct KmvSketch {
    k: usize,
    heap: BinaryHeap<Reverse<u64>>, // max-at-top via Reverse to keep smallest k
}

impl KmvSketch {
    /// Create a KMV sketch keeping k smallest values.
    pub fn new(k: usize) -> Self {
        Self {
            k,
            heap: BinaryHeap::with_capacity(k + 1),
        }
    }

    /// Update with a pre-hashed k-mer value (u64).
    #[inline]
    pub fn update(&mut self, h: u64) {
        if self.heap.len() < self.k {
            self.heap.push(Reverse(h));
        } else if let Some(&Reverse(top)) = self.heap.peek() {
            // "top" is currently the largest among kept minima (due to Reverse)
            if h < top {
                self.heap.pop();
                self.heap.push(Reverse(h));
            }
        }
    }

    /// Produce a fixed-length signature by sorting ascending.
    /// If fewer than k items were observed, the signature will be shorter.
    pub fn finish(mut self) -> Vec<u64> {
        let mut out = Vec::with_capacity(self.heap.len());
        while let Some(Reverse(v)) = self.heap.pop() {
            out.push(v);
        }
        out.sort_unstable();
        out
    }
}
