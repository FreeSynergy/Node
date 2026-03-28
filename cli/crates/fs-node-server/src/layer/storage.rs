// fs-node-server/src/layer/storage.rs — S3Provider: embedded object storage.
//
// Wraps fs-s3 (s3s + s3s-fs + opendal). The Node starts the S3 server
// as a background task; replication backends (Hetzner, SFTP) are opendal layers.

use async_trait::async_trait;
use fs_s3::{S3Server, StorageConfig};
use tokio::task::JoinHandle;
use tracing::info;

use super::NodeLayer;

// ── S3Provider ────────────────────────────────────────────────────────────────

/// Embedded S3-compatible object storage layer.
///
/// Starts `fs-s3` (s3s + s3s-fs) in a background tokio task.
/// Replication to remote targets (Hetzner Storagebox, SFTP) is handled
/// by opendal backends configured in `StorageConfig`.
pub struct S3Provider {
    config: StorageConfig,
}

impl S3Provider {
    /// Create an `S3Provider` from a [`StorageConfig`].
    #[must_use]
    pub fn new(config: StorageConfig) -> Self {
        Self { config }
    }

    /// Start the S3 server in a background task.
    ///
    /// Returns a `JoinHandle` — drop it to abort, or `.await` to join.
    ///
    /// # Errors
    ///
    /// Returns an error if bucket initialization fails or the port is already in use.
    pub async fn start_server(&self) -> anyhow::Result<JoinHandle<()>> {
        S3Server::new(self.config.clone()).start().await
    }

    /// The S3 endpoint this provider will bind (e.g. `http://127.0.0.1:9000`).
    #[must_use]
    pub fn endpoint(&self) -> String {
        format!("http://{}:{}", self.config.bind, self.config.port)
    }
}

#[async_trait]
impl NodeLayer for S3Provider {
    fn name(&self) -> &'static str {
        "s3-provider"
    }

    async fn start(&self) -> anyhow::Result<()> {
        // `NodeServer` calls `start_server()` directly to capture the handle.
        // This no-op satisfies the NodeLayer contract for uniform layer iteration.
        info!(endpoint = %self.endpoint(), "S3Provider ready");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        // The JoinHandle owned by NodeServer is aborted on drop.
        info!("S3Provider stopped");
        Ok(())
    }
}
