// fs-auth/src/permission.rs — Permission, PermissionSet, Role, AccessControl

use serde::{Deserialize, Serialize};
use std::fmt;

// ── Permission ────────────────────────────────────────────────────────────────

/// A single permission string (e.g. `"node:deploy"`, `"node:read"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Permission(pub String);

impl Permission {
    /// Create a new permission from any string-like value.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the permission string as a `&str`.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

// ── PermissionSet ─────────────────────────────────────────────────────────────

/// A set of permissions granted to a principal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    grants: Vec<Permission>,
}

impl PermissionSet {
    /// Build a `PermissionSet` from an iterator of [`Permission`] values.
    pub fn new(grants: impl IntoIterator<Item = Permission>) -> Self {
        Self {
            grants: grants.into_iter().collect(),
        }
    }

    /// `true` when the set contains `permission`.
    pub fn has(&self, permission: &Permission) -> bool {
        self.grants.contains(permission)
    }

    /// `true` when the set contains a permission matching `permission` by string value.
    pub fn has_str(&self, permission: &str) -> bool {
        self.grants.iter().any(|p| p.as_str() == permission)
    }

    /// Add `permission` to the set.
    pub fn grant(&mut self, permission: Permission) {
        if !self.has(&permission) {
            self.grants.push(permission);
        }
    }

    /// Remove `permission` from the set (no-op if absent).
    pub fn revoke(&mut self, permission: &Permission) {
        self.grants.retain(|p| p != permission);
    }

    /// Iterate over all granted permissions.
    pub fn iter(&self) -> impl Iterator<Item = &Permission> {
        self.grants.iter()
    }

    /// `true` when no permissions have been granted.
    pub fn is_empty(&self) -> bool {
        self.grants.is_empty()
    }

    /// `true` when the set contains an exact match **or** a wildcard match.
    ///
    /// Wildcard rules:
    /// - `"*"` matches any single-segment permission (e.g. `"*"` matches `"node:deploy"`).
    /// - `"node:*"` matches any permission that starts with `"node:"`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use fs_auth::{Permission, PermissionSet};
    ///
    /// let ps = PermissionSet::new([Permission::new("node:*")]);
    /// assert!(ps.has_with_wildcard(&Permission::new("node:deploy")));
    /// assert!(ps.has_with_wildcard(&Permission::new("node:read")));
    /// assert!(!ps.has_with_wildcard(&Permission::new("bot:status")));
    /// ```
    pub fn has_with_wildcard(&self, permission: &Permission) -> bool {
        let target = permission.as_str();
        self.grants.iter().any(|p| {
            let pat = p.as_str();
            // Exact match
            if pat == target {
                return true;
            }
            // Global wildcard
            if pat == "*" {
                return true;
            }
            // Prefix wildcard: "node:*" matches "node:deploy"
            if let Some(prefix) = pat.strip_suffix(":*") {
                if let Some(target_prefix) = target.split(':').next() {
                    return prefix == target_prefix;
                }
            }
            false
        })
    }
}

// ── Role ──────────────────────────────────────────────────────────────────────

/// A named role carrying a [`PermissionSet`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    /// Human-readable role name (e.g. `"admin"`, `"deployer"`).
    pub name: String,
    /// Permissions granted to holders of this role.
    pub permissions: PermissionSet,
}

impl Role {
    /// Create a new role with the given name and permission set.
    pub fn new(name: impl Into<String>, permissions: PermissionSet) -> Self {
        Self {
            name: name.into(),
            permissions,
        }
    }
}

// ── AccessControl ─────────────────────────────────────────────────────────────

/// Evaluate whether a principal has access to a given [`Permission`].
///
/// Implement this trait on any type that carries authorization data
/// (e.g. [`PermissionSet`], [`crate::claims::Claims`]).
pub trait AccessControl {
    /// Return `true` if this principal is allowed to exercise `permission`.
    fn allowed(&self, permission: &Permission) -> bool;
}

impl AccessControl for PermissionSet {
    fn allowed(&self, permission: &Permission) -> bool {
        self.has(permission)
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grant_and_has() {
        let mut ps = PermissionSet::default();
        let p = Permission::new("node:deploy");
        ps.grant(p.clone());
        assert!(ps.has(&p));
        assert!(ps.has_str("node:deploy"));
        assert!(!ps.has_str("node:read"));
    }

    #[test]
    fn revoke() {
        let p = Permission::new("node:deploy");
        let mut ps = PermissionSet::new([p.clone()]);
        ps.revoke(&p);
        assert!(ps.is_empty());
    }

    #[test]
    fn access_control_impl() {
        let p = Permission::new("node:deploy");
        let ps = PermissionSet::new([p.clone()]);
        assert!(ps.allowed(&p));
        assert!(!ps.allowed(&Permission::new("node:admin")));
    }

    #[test]
    fn wildcard_global() {
        let ps = PermissionSet::new([Permission::new("*")]);
        assert!(ps.has_with_wildcard(&Permission::new("node:deploy")));
        assert!(ps.has_with_wildcard(&Permission::new("bot:status")));
    }

    #[test]
    fn wildcard_prefix() {
        let ps = PermissionSet::new([Permission::new("node:*")]);
        assert!(ps.has_with_wildcard(&Permission::new("node:deploy")));
        assert!(ps.has_with_wildcard(&Permission::new("node:read")));
        assert!(!ps.has_with_wildcard(&Permission::new("bot:status")));
    }

    #[test]
    fn wildcard_exact_still_works() {
        let ps = PermissionSet::new([Permission::new("node:deploy")]);
        assert!(ps.has_with_wildcard(&Permission::new("node:deploy")));
        assert!(!ps.has_with_wildcard(&Permission::new("node:read")));
    }

    #[test]
    fn no_duplicates_on_grant() {
        let mut ps = PermissionSet::default();
        let p = Permission::new("x");
        ps.grant(p.clone());
        ps.grant(p.clone());
        assert_eq!(ps.grants.len(), 1);
    }
}
