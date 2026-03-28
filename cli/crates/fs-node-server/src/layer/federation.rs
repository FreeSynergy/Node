// fs-node-server/src/layer/federation.rs — FederationGate: inter-node federation.
//
// Placeholder for the federation phase (Phase P).
// Exposes well-known endpoints (.well-known/nodeinfo, .well-known/host-meta)
// so other nodes can discover this one.
//
// Full `ActivityPub` + federated bus routing will be wired here after Phase P.

use async_trait::async_trait;
use fs_federation::well_known::WellKnownPath;
use tracing::info;

use super::NodeLayer;

// ── FederationConfig ──────────────────────────────────────────────────────────

/// Configuration for the federation gate.
#[derive(Debug, Clone)]
pub struct FederationConfig {
    /// Public domain this node is reachable under (e.g. `"node.example.com"`).
    pub domain: String,
    /// Whether to expose `ActivityPub` endpoints (requires Phase P).
    pub activitypub_enabled: bool,
}

impl FederationConfig {
    /// Minimal config: domain only, `ActivityPub` disabled.
    #[must_use]
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            activitypub_enabled: false,
        }
    }
}

// ── FederationGate ────────────────────────────────────────────────────────────

/// Federation orchestration layer.
///
/// Currently provides well-known URL builders for discovery.
/// Full federation (`ActivityPub`, federated bus) is wired in Phase P.
pub struct FederationGate {
    config: FederationConfig,
}

impl FederationGate {
    /// Create a `FederationGate` for `config.domain`.
    #[must_use]
    pub fn new(config: FederationConfig) -> Self {
        Self { config }
    }

    /// `.well-known/nodeinfo` URL for this node.
    #[must_use]
    pub fn nodeinfo_url(&self) -> String {
        WellKnownPath::url(&self.config.domain, WellKnownPath::NODEINFO)
    }

    /// `.well-known/host-meta` URL for this node.
    #[must_use]
    pub fn host_meta_url(&self) -> String {
        WellKnownPath::url(&self.config.domain, WellKnownPath::HOST_META)
    }

    /// Public domain this gate is configured for.
    #[must_use]
    pub fn domain(&self) -> &str {
        &self.config.domain
    }
}

#[async_trait]
impl NodeLayer for FederationGate {
    fn name(&self) -> &'static str {
        "federation-gate"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!(domain = %self.config.domain, "FederationGate started (Phase-P stub)");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!("FederationGate stopped");
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn well_known_urls_contain_domain() {
        let gate = FederationGate::new(FederationConfig::new("node.example.com"));
        assert!(
            gate.nodeinfo_url().contains("node.example.com"),
            "nodeinfo must contain domain"
        );
        assert!(
            gate.host_meta_url().contains("node.example.com"),
            "host-meta must contain domain"
        );
    }
}
