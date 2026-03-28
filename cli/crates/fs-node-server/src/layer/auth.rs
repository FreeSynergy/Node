// fs-node-server/src/layer/auth.rs — AuthGateway: auth protocol facade.
//
// AuthGateway is the Node's single entry point for authentication.
// It delegates to fs-auth protocol traits (OAuthProvider, SsoProvider, etc.)
// so any backend (Kanidm, Keycloak, …) can be swapped without changing Node code.
//
// G1.2 will plug KanidmBackend here; until then callers compose an AuthGateway
// from any OAuthProvider + SsoProvider implementation.

use async_trait::async_trait;
use fs_auth::{AuthCapabilities, OAuthProvider, SsoProvider};
use tracing::info;

use super::NodeLayer;

// ── AuthGateway ───────────────────────────────────────────────────────────────

/// Auth orchestration layer.
///
/// Holds boxed protocol-trait implementations. Switch backends by providing
/// different implementations — the Node API remains unchanged.
pub struct AuthGateway {
    backend_name: String,
    capabilities: AuthCapabilities,
    oauth: Box<dyn OAuthProvider>,
    sso: Box<dyn SsoProvider>,
}

impl AuthGateway {
    /// Create a new `AuthGateway` from a named backend and its protocol impls.
    pub fn new(
        backend_name: impl Into<String>,
        capabilities: AuthCapabilities,
        oauth: Box<dyn OAuthProvider>,
        sso: Box<dyn SsoProvider>,
    ) -> Self {
        Self {
            backend_name: backend_name.into(),
            capabilities,
            oauth,
            sso,
        }
    }

    /// The name of the configured auth backend (e.g. `"kanidm"`).
    #[must_use]
    pub fn backend_name(&self) -> &str {
        &self.backend_name
    }

    /// Which protocols the configured backend supports.
    #[must_use]
    pub fn capabilities(&self) -> &AuthCapabilities {
        &self.capabilities
    }

    /// Access the OAuth 2.0 / OIDC provider.
    #[must_use]
    pub fn oauth(&self) -> &dyn OAuthProvider {
        self.oauth.as_ref()
    }

    /// Access the SSO session provider.
    #[must_use]
    pub fn sso(&self) -> &dyn SsoProvider {
        self.sso.as_ref()
    }
}

#[async_trait]
impl NodeLayer for AuthGateway {
    fn name(&self) -> &'static str {
        "auth-gateway"
    }

    async fn start(&self) -> anyhow::Result<()> {
        info!(backend = %self.backend_name, "AuthGateway started");
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        info!(backend = %self.backend_name, "AuthGateway stopped");
        Ok(())
    }
}
