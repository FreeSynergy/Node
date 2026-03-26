// fs-auth/src/claims.rs — JWT Claims + AccessControl impl

use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::permission::{AccessControl, Permission, PermissionSet};

// ── Claims ────────────────────────────────────────────────────────────────────

/// Standard JWT claims used across FreeSynergy services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    /// Subject — user ID or service ID.
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Audience.
    pub aud: String,
    /// Expiry as Unix timestamp (seconds since epoch).
    pub exp: u64,
    /// Issued-at as Unix timestamp (seconds since epoch).
    pub iat: u64,
    /// Permissions granted to this principal.
    pub permissions: PermissionSet,
}

impl Claims {
    /// Create claims that expire `ttl_seconds` from now.
    ///
    /// `iat` is set to the current Unix time; `exp` is `iat + ttl_seconds`.
    pub fn new(
        sub: impl Into<String>,
        iss: impl Into<String>,
        aud: impl Into<String>,
        ttl_seconds: u64,
        permissions: PermissionSet,
    ) -> Self {
        let iat = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            sub: sub.into(),
            iss: iss.into(),
            aud: aud.into(),
            exp: iat + ttl_seconds,
            iat,
            permissions,
        }
    }

    /// `true` when the token has expired (current Unix time > `exp`).
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now > self.exp
    }
}

impl AccessControl for Claims {
    fn allowed(&self, permission: &Permission) -> bool {
        self.permissions.allowed(permission)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_expired_with_large_ttl() {
        let claims = Claims::new("user:1", "fsn", "api", 3600, PermissionSet::default());
        assert!(!claims.is_expired());
    }

    #[test]
    fn expired_with_zero_ttl() {
        let mut claims = Claims::new("user:1", "fsn", "api", 0, PermissionSet::default());
        // Force exp into the past.
        claims.exp = 0;
        assert!(claims.is_expired());
    }

    #[test]
    fn access_control_delegation() {
        let p = Permission::new("node:deploy");
        let ps = PermissionSet::new([p.clone()]);
        let claims = Claims::new("user:1", "fsn", "api", 3600, ps);
        assert!(claims.allowed(&p));
        assert!(!claims.allowed(&Permission::new("node:admin")));
    }
}
