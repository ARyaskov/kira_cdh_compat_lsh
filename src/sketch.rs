//! Sketch utilities common to MinHash and KMV.

/// Compute a MinHash-style Jaccard estimate from two signatures of equal length.
/// This function treats signatures as MinHash-like: equality per position.
pub fn jaccard_from_signatures(a: &[u64], b: &[u64]) -> f64 {
    let n = a.len().min(b.len());
    if n == 0 {
        return 0.0;
    }
    let mut eq = 0usize;
    for i in 0..n {
        if a[i] == b[i] {
            eq += 1;
        }
    }
    eq as f64 / n as f64
}
