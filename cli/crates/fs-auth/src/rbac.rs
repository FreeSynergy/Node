// fs-auth/src/rbac.rs — Role-Based Access Control.

use std::collections::HashMap;

use crate::permission::{Permission, PermissionSet, Role};

// ── Rbac trait ────────────────────────────────────────────────────────────────

/// Role-based access control — maps subjects to roles and permissions.
///
/// Implement this trait to integrate with any role store (in-memory, database,
/// OIDC claims, …).
pub trait Rbac: Send + Sync {
    /// Return all role names assigned to `subject`.
    fn roles_for(&self, subject: &str) -> Vec<String>;

    /// Return the merged [`PermissionSet`] for `subject` (across all their roles).
    fn permissions_for(&self, subject: &str) -> PermissionSet;

    /// `true` when `subject` holds `permission` (exact or wildcard match).
    fn check(&self, subject: &str, permission: &Permission) -> bool {
        self.permissions_for(subject).has_with_wildcard(permission)
    }
}

// ── InMemoryRbac ──────────────────────────────────────────────────────────────

/// An in-memory [`Rbac`] implementation backed by `HashMap`s.
///
/// Suitable for configuration-driven setups or tests.
///
/// # Example
///
/// ```rust
/// use fs_auth::{Permission, PermissionSet, Role, InMemoryRbac, Rbac};
///
/// let mut rbac = InMemoryRbac::new();
///
/// rbac.define_role(Role::new("deployer", PermissionSet::new([
///     Permission::new("node:deploy"),
///     Permission::new("node:read"),
/// ])));
///
/// rbac.assign("alice", "deployer");
///
/// assert!(rbac.check("alice", &Permission::new("node:deploy")));
/// assert!(!rbac.check("alice", &Permission::new("node:admin")));
/// ```
#[derive(Default)]
pub struct InMemoryRbac {
    /// `subject → [role_name, …]`
    assignments: HashMap<String, Vec<String>>,
    /// `role_name → Role`
    role_definitions: HashMap<String, Role>,
}

impl InMemoryRbac {
    /// Create an empty RBAC store.
    pub fn new() -> Self {
        Self::default()
    }

    /// Define a role. Replaces any existing role with the same name.
    pub fn define_role(&mut self, role: Role) {
        self.role_definitions.insert(role.name.clone(), role);
    }

    /// Assign `subject` to `role_name`.
    ///
    /// The role must be defined via [`define_role`](InMemoryRbac::define_role)
    /// before subjects can be checked against it.
    pub fn assign(&mut self, subject: impl Into<String>, role_name: impl Into<String>) {
        let roles = self.assignments.entry(subject.into()).or_default();
        let name = role_name.into();
        if !roles.contains(&name) {
            roles.push(name);
        }
    }

    /// Remove `subject` from `role_name`.
    pub fn unassign(&mut self, subject: &str, role_name: &str) {
        if let Some(roles) = self.assignments.get_mut(subject) {
            roles.retain(|r| r != role_name);
        }
    }
}

impl Rbac for InMemoryRbac {
    fn roles_for(&self, subject: &str) -> Vec<String> {
        self.assignments.get(subject).cloned().unwrap_or_default()
    }

    fn permissions_for(&self, subject: &str) -> PermissionSet {
        let mut merged = PermissionSet::default();
        for role_name in self.roles_for(subject) {
            if let Some(role) = self.role_definitions.get(&role_name) {
                for perm in role.permissions.iter() {
                    merged.grant(perm.clone());
                }
            }
        }
        merged
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::{Permission, PermissionSet, Role};

    fn make_rbac() -> InMemoryRbac {
        let mut rbac = InMemoryRbac::new();
        rbac.define_role(Role::new(
            "admin",
            PermissionSet::new([Permission::new("node:*")]),
        ));
        rbac.define_role(Role::new(
            "viewer",
            PermissionSet::new([Permission::new("node:read")]),
        ));
        rbac.assign("alice", "admin");
        rbac.assign("bob", "viewer");
        rbac
    }

    #[test]
    fn admin_can_deploy() {
        let rbac = make_rbac();
        assert!(rbac.check("alice", &Permission::new("node:deploy")));
    }

    #[test]
    fn viewer_can_only_read() {
        let rbac = make_rbac();
        assert!(rbac.check("bob", &Permission::new("node:read")));
        assert!(!rbac.check("bob", &Permission::new("node:deploy")));
    }

    #[test]
    fn unknown_subject_has_no_permissions() {
        let rbac = make_rbac();
        assert!(!rbac.check("unknown", &Permission::new("node:read")));
    }

    #[test]
    fn roles_for_returns_assigned_roles() {
        let rbac = make_rbac();
        let roles = rbac.roles_for("alice");
        assert!(roles.contains(&"admin".to_string()));
    }

    #[test]
    fn unassign_removes_role() {
        let mut rbac = make_rbac();
        rbac.unassign("alice", "admin");
        assert!(!rbac.check("alice", &Permission::new("node:deploy")));
    }

    #[test]
    fn multiple_roles_merge_permissions() {
        let mut rbac = make_rbac();
        rbac.assign("carol", "admin");
        rbac.assign("carol", "viewer");
        assert!(rbac.check("carol", &Permission::new("node:deploy")));
        assert!(rbac.check("carol", &Permission::new("node:read")));
    }
}
