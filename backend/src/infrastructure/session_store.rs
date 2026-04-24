//! Filesystem-backed WA session blob helpers.
//!
//! Atomic persist pattern: write to `session.bin.tmp`, fsync, rename.
//! On load: missing/corrupt → warn + return `None`, caller re-pairs.

use std::path::{Path, PathBuf};

use tokio::fs;
use tokio::io::AsyncWriteExt;

/// Canonical filename for the on-disk WA session blob.
pub const SESSION_FILE: &str = "session.bin";
const SESSION_TMP: &str = "session.bin.tmp";

/// Ensure `dir` exists (recursive). Idempotent.
pub async fn ensure_dir(dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dir).await
}

/// Read the WA session blob from `dir` if it exists + is readable.
pub async fn load(dir: &Path) -> Option<Vec<u8>> {
    let path = session_path(dir);
    match fs::read(&path).await {
        Ok(bytes) => Some(bytes),
        Err(e) => {
            tracing::warn!(path = %path.display(), error = %e, "wa session load failed");
            None
        }
    }
}

/// Persist `bytes` to the WA session blob atomically.
pub async fn save(dir: &Path, bytes: &[u8]) -> std::io::Result<()> {
    ensure_dir(dir).await?;
    let tmp = dir.join(SESSION_TMP);
    {
        let mut f = fs::File::create(&tmp).await?;
        f.write_all(bytes).await?;
        f.sync_all().await?;
    }
    fs::rename(&tmp, session_path(dir)).await?;
    Ok(())
}

/// Path to the final session file within `dir`.
pub fn session_path(dir: &Path) -> PathBuf {
    dir.join(SESSION_FILE)
}
