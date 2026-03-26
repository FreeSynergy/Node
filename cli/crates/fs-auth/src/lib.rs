// fs-auth — Permission system, JWT claims, and JWT signing/verification
// for the FreeSynergy ecosystem.
//
// # Modules
//
// - [`permission`] — [`Permission`], [`PermissionSet`], [`Role`], [`AccessControl`] trait
// - [`claims`]     — [`Claims`] (standard JWT payload with permissions)
// - [`jwt`]        — [`JwtSigner`] / [`JwtValidator`] (feature `"jwt"`)
//
// # Features
//
// | feature | adds                              | extra dep         |
// |---------|-----------------------------------|-------------------|
// | `jwt`   | `JwtSigner`, `JwtValidator`       | `jsonwebtoken = "9"` |

pub mod claims;
pub mod permission;
pub mod rbac;

#[cfg(feature = "jwt")]
pub mod jwt;

#[cfg(feature = "oauth2")]
pub mod oauth2;

// ── Flat re-exports ───────────────────────────────────────────────────────────

pub use claims::Claims;
pub use permission::{AccessControl, Permission, PermissionSet, Role};
pub use rbac::{InMemoryRbac, Rbac};

#[cfg(feature = "jwt")]
pub use jwt::{JwtAlgorithm, JwtSigner, JwtValidator};

#[cfg(feature = "oauth2")]
pub use oauth2::{OAuth2Client, OAuth2Config, TokenResponse};
