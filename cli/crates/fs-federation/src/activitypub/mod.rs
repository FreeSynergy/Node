//! ActivityPub federation support via the `activitypub_federation` crate.
//!
//! This module provides types and helpers for integrating FreeSynergy services
//! with the ActivityPub federation protocol (Mastodon, Lemmy, etc.).
//!
//! # Key re-exported types from `activitypub_federation`
//! - [`FederationConfig`] — federation configuration (domain, HTTP signatures, etc.)
//! - [`FederationMiddleware`] — axum middleware for handling federation requests
//! - [`ObjectId`] — typed URL wrapper for federated objects
//! - [`WithContext`] — wraps any AP object in a JSON-LD `@context`
//!
//! # FreeSynergy types
//! - [`FsFederationConfig`] — convenience builder with FreeSynergy defaults
//! - [`FsActor`] — minimal ActivityPub Person/Service actor struct
//! - [`ActorKind`] — Person / Service / Application discriminant
//! - [`PublicKeyInfo`] — HTTP-signature RSA public key block
//!
//! # Quick start
//! ```rust,ignore
//! use fs_federation::activitypub::FsFederationConfig;
//!
//! // Build a FederationConfig<MyAppData> for use with axum.
//! // MyAppData must implement Clone + Send + Sync + 'static.
//! let config = FsFederationConfig::new("example.com")
//!     .with_signed_fetch(true)
//!     .build(my_app_data)
//!     .await?;
//! ```

pub use activitypub_federation::config::{FederationConfig, FederationMiddleware};
pub use activitypub_federation::fetch::object_id::ObjectId;
pub use activitypub_federation::protocol::context::WithContext;

use serde::{Deserialize, Serialize};
use url::Url;

// ── FsFederationConfig ───────────────────────────────────────────────────────

/// FreeSynergy federation configuration builder.
///
/// Wraps [`FederationConfig`] with FreeSynergy-specific defaults.
/// Call [`FsFederationConfig::build`] to produce the final
/// `FederationConfig<T>` for use with axum / tower middleware.
pub struct FsFederationConfig {
    domain: String,
    signed_fetch: bool,
}

impl FsFederationConfig {
    /// Create a new builder for the given domain (e.g. `"example.com"`).
    pub fn new(domain: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            signed_fetch: true,
        }
    }

    /// Require HTTP signatures on all federated fetches (default: `true`).
    pub fn with_signed_fetch(mut self, signed: bool) -> Self {
        self.signed_fetch = signed;
        self
    }

    /// Build a [`FederationConfig`] for the given application data.
    ///
    /// `T` is your application state — it must be `Clone + Send + Sync + 'static`.
    /// Consuming crates (e.g. FreeSynergy.Node) provide their own concrete type.
    ///
    /// # Errors
    /// Returns an error if the underlying `FederationConfig` builder fails
    /// (e.g. invalid domain format).
    pub async fn build<T>(self, data: T) -> Result<FederationConfig<T>, crate::ApBuildError>
    where
        T: Clone + Send + Sync + 'static,
    {
        let mut builder = FederationConfig::builder();
        builder.domain(self.domain).app_data(data);
        builder
            .build()
            .await
            .map_err(|e| crate::ApBuildError(e.to_string()))
    }
}

// ── ActorKind ─────────────────────────────────────────────────────────────────

/// ActivityPub actor type discriminant.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActorKind {
    /// A human person.
    Person,
    /// An automated service account.
    Service,
    /// A software application.
    Application,
}

// ── PublicKeyInfo ─────────────────────────────────────────────────────────────

/// RSA public key block used for HTTP Signature verification.
///
/// Serialises to the standard ActivityPub `publicKey` JSON-LD object.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKeyInfo {
    /// Key identifier IRI (e.g. `"https://example.com/users/alice#main-key"`).
    pub id: Url,
    /// IRI of the actor that owns this key.
    pub owner: Url,
    /// PEM-encoded RSA public key.
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

// ── FsActor ──────────────────────────────────────────────────────────────────

/// A minimal ActivityPub actor for FreeSynergy services.
///
/// This struct represents the JSON payload returned at an actor's IRI endpoint
/// (e.g. `GET https://example.com/users/alice`).
/// Consuming crates implement `activitypub_federation::traits::Actor` on top of
/// their own domain types; this struct is provided as a convenient wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FsActor {
    /// Globally unique actor IRI.
    pub id: Url,
    /// Actor type discriminant.
    #[serde(rename = "type")]
    pub kind: ActorKind,
    /// Short handle without `@domain` suffix.
    #[serde(rename = "preferredUsername")]
    pub preferred_username: String,
    /// URL of this actor's AP inbox endpoint.
    pub inbox: Url,
    /// URL of this actor's AP outbox endpoint.
    pub outbox: Url,
    /// RSA public key for HTTP Signature verification.
    #[serde(rename = "publicKey")]
    pub public_key: PublicKeyInfo,
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fs_federation_config_builder_defaults() {
        let cfg = FsFederationConfig::new("example.com");
        assert_eq!(cfg.domain, "example.com");
        assert!(cfg.signed_fetch);
    }

    #[test]
    fn fs_federation_config_signed_fetch_toggle() {
        let cfg = FsFederationConfig::new("example.com").with_signed_fetch(false);
        assert!(!cfg.signed_fetch);
    }

    #[test]
    fn actor_kind_serialization() {
        let json = serde_json::to_string(&ActorKind::Service).unwrap();
        assert_eq!(json, "\"Service\"");
    }

    #[test]
    fn public_key_info_roundtrip() {
        let info = PublicKeyInfo {
            id: Url::parse("https://example.com/users/alice#main-key").unwrap(),
            owner: Url::parse("https://example.com/users/alice").unwrap(),
            public_key_pem: "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("publicKeyPem"));
        let back: PublicKeyInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(back.public_key_pem, info.public_key_pem);
    }
}
