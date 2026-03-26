// fs-federation/src/oidc/mod.rs — OIDC discovery + token validation

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── OidcConfig ────────────────────────────────────────────────────────────────

/// Configuration for an OIDC provider connection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    /// OIDC issuer URL (e.g. `"https://auth.example.com"`).
    pub issuer: String,
    /// OAuth2 client identifier.
    pub client_id: String,
    /// OAuth2 client secret.
    pub client_secret: String,
}

// ── OidcDiscovery ─────────────────────────────────────────────────────────────

/// Subset of the OIDC `.well-known/openid-configuration` discovery document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcDiscovery {
    /// Issuer identifier.
    pub issuer: String,
    /// Authorization endpoint URL.
    pub authorization_endpoint: String,
    /// Token endpoint URL.
    pub token_endpoint: String,
    /// Userinfo endpoint URL (optional).
    pub userinfo_endpoint: Option<String>,
    /// JSON Web Key Set URI.
    pub jwks_uri: String,
    /// Token introspection endpoint (optional).
    pub introspection_endpoint: Option<String>,
}

// ── OidcClaims ────────────────────────────────────────────────────────────────

/// Claims returned from an OIDC userinfo or token response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcClaims {
    /// Subject identifier.
    pub sub: String,
    /// User email address.
    pub email: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Preferred username.
    pub preferred_username: Option<String>,
    /// Group memberships.
    pub groups: Option<Vec<String>>,
}

// ── OidcClient ────────────────────────────────────────────────────────────────

/// OIDC client for validating tokens against an OIDC provider.
pub struct OidcClient {
    config: OidcConfig,
    http: reqwest::Client,
}

impl OidcClient {
    /// Create a new [`OidcClient`] with the given configuration.
    pub fn new(config: OidcConfig) -> Self {
        Self {
            config,
            http: reqwest::Client::new(),
        }
    }

    /// Fetch the OIDC discovery document from
    /// `{issuer}/.well-known/openid-configuration`.
    pub async fn discover(&self) -> Result<OidcDiscovery, FsError> {
        let url = format!(
            "{}/.well-known/openid-configuration",
            self.config.issuer.trim_end_matches('/')
        );
        let response = self
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| FsError::network(format!("OIDC discovery request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(FsError::network(format!(
                "OIDC discovery returned HTTP {}",
                response.status()
            )));
        }

        response
            .json::<OidcDiscovery>()
            .await
            .map_err(|e| FsError::parse(format!("OIDC discovery JSON parse failed: {e}")))
    }

    /// Fetch userinfo for the given bearer token from the provider's userinfo endpoint.
    pub async fn userinfo(
        &self,
        discovery: &OidcDiscovery,
        bearer_token: &str,
    ) -> Result<OidcClaims, FsError> {
        let endpoint = discovery
            .userinfo_endpoint
            .as_deref()
            .ok_or_else(|| FsError::config("OIDC provider does not expose a userinfo endpoint"))?;

        let response = self
            .http
            .get(endpoint)
            .bearer_auth(bearer_token)
            .send()
            .await
            .map_err(|e| FsError::network(format!("OIDC userinfo request failed: {e}")))?;

        if !response.status().is_success() {
            return Err(FsError::auth(format!(
                "OIDC userinfo returned HTTP {}",
                response.status()
            )));
        }

        response
            .json::<OidcClaims>()
            .await
            .map_err(|e| FsError::parse(format!("OIDC userinfo JSON parse failed: {e}")))
    }
}
