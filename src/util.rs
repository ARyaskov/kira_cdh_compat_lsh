//! Small utilities: splitmix64 mixing and helpers.

#[inline]
pub fn splitmix64(mut x: u64) -> u64 {
    // From Steele et al. (SplitMix64) – deterministic across platforms.
    x = x.wrapping_add(0x9E3779B97F4A7C15);
    let mut z = x;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
    z ^ (z >> 31)
}

#[inline]
pub fn mix_with_seed(x: u64, seed: u64) -> u64 {
    // Deterministic "permutation": feed x^seed through splitmix64.
    splitmix64(x ^ seed)
}

#[inline]
pub fn hash_band(chunk: &[u64], seed: u64) -> u64 {
    // Fold a small chunk of u64 values into one 64-bit signature deterministically.
    // Avoid hashing libs to keep it blazing fast and fully deterministic.
    let mut acc = seed ^ 0xDEADBEEFDEADBEEF;
    for &v in chunk {
        acc = splitmix64(acc ^ v);
    }
    acc
}
