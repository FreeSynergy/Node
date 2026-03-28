// fs-node-server/src/layer/mod.rs — Node orchestration layers.
//
// Each layer wraps one subsystem (auth, storage, proxy, federation, api).
// All layers share the NodeLayer lifecycle trait: start / stop / name.

pub mod auth;
pub mod federation;
pub mod proxy;
pub mod storage;

pub use auth::AuthGateway;
pub use federation::FederationGate;
pub use proxy::ServiceProxy;
pub use storage::S3Provider;

use async_trait::async_trait;

/// Common lifecycle interface for every Node orchestration layer.
#[async_trait]
pub trait NodeLayer: Send + Sync {
    /// Stable identifier used in logs and health output.
    fn name(&self) -> &'static str;

    /// Start the layer: open connections, bind ports, spawn tasks.
    async fn start(&self) -> anyhow::Result<()>;

    /// Gracefully stop the layer.
    async fn stop(&self) -> anyhow::Result<()>;
}
