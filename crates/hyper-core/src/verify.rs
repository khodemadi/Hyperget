use crate::{Error, Result};
use sha2::{Digest, Sha256};
use std::path::Path;
use tokio::io::AsyncReadExt;
pub async fn sha256(path: &Path, expected: &str) -> Result<()> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut h = Sha256::new();
    let mut b = vec![0; 128 * 1024];
    loop {
        let n = f.read(&mut b).await?;
        if n == 0 {
            break;
        }
        h.update(&b[..n]);
    }
    let actual = format!("{:x}", h.finalize());
    if actual.eq_ignore_ascii_case(expected) {
        Ok(())
    } else {
        Err(Error::Verification(format!("expected {expected}, got {actual}")))
    }
}
