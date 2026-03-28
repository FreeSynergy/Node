// fs-node-server/src/invite — Node invitation system (G1.6).
//
// Modules:
//   token   — InviteToken: crypto-signed, time-limited invite string
//   bundle  — InviteBundle: age-encrypted TOML connection package
//   ports   — PortPool: per-invite TCP port allocation

pub mod bundle;
pub mod ports;
pub mod token;

pub use bundle::InviteBundle;
pub use ports::PortPool;
pub use token::InviteToken;

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::error::NodeServerError;

// ── InviteSystem ──────────────────────────────────────────────────────────────

/// High-level facade for the invite flow.
///
/// Combines token generation, bundle encryption, and port allocation.
/// Callers use this instead of wiring the three sub-modules manually.
pub struct InviteSystem {
    /// Node-local HMAC secret for token signing.
    secret: Vec<u8>,
    /// Port range for dedicated invite connections.
    ports: PortPool,
    /// Public base URL of this node (e.g. `"https://node.example.com"`).
    node_url: String,
}

impl InviteSystem {
    /// Create a new `InviteSystem`.
    ///
    /// - `secret`: at least 32 random bytes, kept in node config.
    /// - `port_min` / `port_max`: dedicated invite port range `[min, max)`.
    /// - `node_url`: the public URL advertised in invite bundles.
    #[must_use]
    pub fn new(
        secret: impl Into<Vec<u8>>,
        port_min: u16,
        port_max: u16,
        node_url: impl Into<String>,
    ) -> Self {
        Self {
            secret: secret.into(),
            ports: PortPool::new(port_min, port_max),
            node_url: node_url.into(),
        }
    }

    /// Generate a new invite: token + encrypted bundle + allocated port.
    ///
    /// Returns `(token_string, encrypted_bundle, port)`.
    ///
    /// # Errors
    ///
    /// Returns [`NodeServerError::Invite`] if no ports are available or
    /// bundle encryption fails.
    pub fn create(
        &self,
        ttl_secs: i64,
        label: Option<String>,
    ) -> Result<(String, String, u16), NodeServerError> {
        let port = self
            .ports
            .allocate()
            .ok_or_else(|| NodeServerError::Invite("no invite ports available".into()))?;

        let token = InviteToken::generate(ttl_secs, &self.secret);
        let token_str = token.to_string();

        let bundle = InviteBundle::new(token.id(), &self.node_url, port, token.expires_at(), label);

        match bundle.encrypt(&token_str) {
            Ok(encrypted) => Ok((token_str, encrypted, port)),
            Err(e) => {
                self.ports.release(port);
                Err(e)
            }
        }
    }

    /// Verify a token string and return its ID + expiry on success.
    ///
    /// # Errors
    ///
    /// Returns [`NodeServerError::Invite`] if the token signature is invalid
    /// or the token has expired.
    pub fn verify(&self, token_str: &str) -> Result<(Uuid, DateTime<Utc>), NodeServerError> {
        let token = InviteToken::parse(token_str)
            .map_err(|e| NodeServerError::Invite(format!("parse: {e}")))?;
        token
            .verify(&self.secret)
            .map_err(|e| NodeServerError::Invite(format!("verify: {e}")))?;
        Ok((token.id(), token.expires_at()))
    }

    /// Decrypt an invite bundle using the provided token string as the passphrase.
    ///
    /// # Errors
    ///
    /// Returns [`NodeServerError`] if decryption fails.
    pub fn open_bundle(
        &self,
        token_str: &str,
        encrypted_bundle: &str,
    ) -> Result<InviteBundle, NodeServerError> {
        InviteBundle::decrypt(encrypted_bundle, token_str)
    }

    /// Release the port associated with an invite (call on expiry or acceptance).
    pub fn release_port(&self, port: u16) {
        self.ports.release(port);
    }

    /// Number of invite ports still available.
    #[must_use]
    pub fn available_ports(&self) -> usize {
        self.ports.available_count()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn system() -> InviteSystem {
        InviteSystem::new(
            b"test-secret-32-bytes-padded!!!!".to_vec(),
            9300,
            9310,
            "https://node.example.com",
        )
    }

    #[test]
    fn create_and_verify() {
        let sys = system();
        let (token_str, bundle_enc, port) = sys.create(3600, Some("test".into())).unwrap();

        let (id, _expiry) = sys.verify(&token_str).unwrap();
        let bundle = sys.open_bundle(&token_str, &bundle_enc).unwrap();

        assert_eq!(bundle.token_id, id);
        assert_eq!(bundle.invite_port, port);
        assert_eq!(bundle.node_url, "https://node.example.com");
    }

    #[test]
    fn exhausted_pool_returns_error() {
        let sys = InviteSystem::new(b"secret".to_vec(), 9400, 9401, "https://node.example.com");
        let _ = sys.create(3600, None).unwrap(); // uses the only port
        assert!(sys.create(3600, None).is_err());
    }

    #[test]
    fn released_port_becomes_available() {
        let sys = InviteSystem::new(b"secret".to_vec(), 9500, 9501, "https://node.example.com");
        let (_, _, port) = sys.create(3600, None).unwrap();
        sys.release_port(port);
        assert_eq!(sys.available_ports(), 1);
    }
}
