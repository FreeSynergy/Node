// fs-node-server/src/layer/proxy.rs — ServiceProxy: capability-based request routing.
//
// ServiceProxy wraps fs-registry and answers:
//   "Which endpoint handles capability X right now?"
//
// Services register their capabilities on startup via the bus; the proxy
// performs the lookup so the Node API can forward requests transparently.

use async_trait::async_trait;
use fs_registry::{Registry, RegistryError, ServiceEntry};
use tracing::info;

use super::NodeLayer;

// ── ServiceProxy ──────────────────────────────────────────────────────────────

/// Service capability routing layer.
///
/// Opens the registry database and provides a synchronous query interface
/// over the registered capabilities.
pub struct ServiceProxy {
    registry: Registry,
}

impl ServiceProxy {
    /// Open (or create) the capability registry database.
    ///
    /// Pass `":memory:"` in tests.
    ///
    /// # Errors
    ///
    /// Returns a [`RegistryError`] if the database cannot be opened.
    pub async fn open(db_path: &str) -> Result<Self, RegistryError> {
        let registry = Registry::open(db_path).await?;
        Ok(Self { registry })
    }

    /// Find the endpoint for the first `Up` service advertising `capability`.
    ///
    /// Returns `None` when no matching service is registered or all are down.
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] on database failure.
    pub async fn endpoint_for(&self, capability: &str) -> Result<Option<String>, RegistryError> {
        self.registry.endpoint_for_capability(capability).await
    }

    /// Register a service capability (called by services on startup).
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] on database failure.
    pub async fn register(&self, entry: ServiceEntry) -> Result<(), RegistryError> {
        self.registry.register(entry).await
    }

    /// Deregister all capabilities of a service (called on shutdown).
    ///
    /// # Errors
    ///
    /// Returns [`RegistryError`] on database failure.
    pub async fn deregister(&self, service_id: &str) -> Result<(), RegistryError> {
        self.registry.deregister(service_id).await
    }
}

#[async_trait]
impl NodeLayer for ServiceProxy {
    fn name(&self) -> &'static str {
        "service-proxy"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!("ServiceProxy started");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("ServiceProxy stopped");
        Ok(())
    }
}
