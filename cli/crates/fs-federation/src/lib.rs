// fs-federation — OIDC discovery, SCIM provisioning, and ActivityPub federation
// for the FreeSynergy ecosystem.
//
// # Modules
//
// | module          | feature       | provides                                                  |
// |-----------------|---------------|-----------------------------------------------------------|
// | [`well_known`]  | always        | `.well-known` URL builders + NodeInfo/HostMeta types      |
// | [`oidc`]        | `oidc`        | OIDC discovery, userinfo, [`OidcClient`]                  |
// | [`scim`]        | `scim`        | SCIM 2.0 user/group types, [`ScimClient`]                 |
// | [`activitypub`] | `activitypub` | AP types + `activitypub_federation` re-exports            |
// | [`webfinger`]   | `webfinger`   | WebFinger RFC 7033 client                                 |
//
// # Features
//
// | feature        | adds                                  | extra deps                          |
// |----------------|---------------------------------------|-------------------------------------|
// | `oidc`         | OIDC discovery + userinfo             | `reqwest`, `tokio`                  |
// | `scim`         | SCIM 2.0 provisioning client          | `reqwest`, `tokio`                  |
// | `activitypub`  | ActivityPub types + federation crate  | `activitypub_federation`, `url`     |
// | `webfinger`    | WebFinger RFC 7033 client             | `reqwest`, `tokio`                  |

pub mod well_known;

#[cfg(feature = "oidc")]
pub mod oidc;

#[cfg(feature = "scim")]
pub mod scim;

#[cfg(feature = "activitypub")]
pub mod activitypub;

#[cfg(feature = "webfinger")]
pub mod webfinger;

// ── ApBuildError ──────────────────────────────────────────────────────────────

/// Error returned when building a [`activitypub::FsFederationConfig`] fails.
///
/// Wraps the string representation of the underlying
/// `activitypub_federation` builder error.
#[cfg(feature = "activitypub")]
#[derive(Debug)]
pub struct ApBuildError(pub String);

#[cfg(feature = "activitypub")]
impl std::fmt::Display for ApBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ActivityPub config build error: {}", self.0)
    }
}

#[cfg(feature = "activitypub")]
impl std::error::Error for ApBuildError {}

// ── Flat re-exports ───────────────────────────────────────────────────────────

#[cfg(feature = "oidc")]
pub use oidc::{OidcClaims, OidcClient, OidcConfig, OidcDiscovery};

#[cfg(feature = "scim")]
pub use scim::{ScimClient, ScimEmail, ScimGroup, ScimMember, ScimUser};

#[cfg(feature = "activitypub")]
pub use activitypub::{
    ActorKind, FederationConfig, FederationMiddleware, FsActor, FsFederationConfig, ObjectId,
    PublicKeyInfo, WithContext,
};

pub use well_known::{HostMeta, HostMetaLink, NodeInfoLink, NodeInfoPointer, WellKnownPath};

#[cfg(feature = "webfinger")]
pub use webfinger::{WebFingerClient, WebFingerLink, WebFingerResponse};
