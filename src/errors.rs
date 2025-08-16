use thiserror::Error;

#[derive(Debug, Error)]
pub enum LshError {
    #[error(
        "invalid LSH params: bands * rows must equal signature length (got bands={bands}, rows={rows}, sig_len={sig_len})"
    )]
    InvalidParams {
        bands: usize,
        rows: usize,
        sig_len: usize,
    },

    #[error("signature length {sig_len} is smaller than bands*rows={need}")]
    ShortSignature { sig_len: usize, need: usize },
}
