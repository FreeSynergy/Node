// fs-node-server/src/invite/bundle.rs — InviteBundle: encrypted TOML connection package.
//
// An InviteBundle contains everything the invited node needs to connect:
//   - The inviting node's public URL
//   - The dedicated port allocated for this invite
//   - The token ID (for server-side lookup)
//   - Expiry timestamp
//
// The bundle is serialised as TOML and then encrypted with `age`
// (scrypt + passphrase = the raw invite token string).
//
// Wire format: age-encrypted armored text (ASCII, safe to paste in a terminal).

use std::io::{Read, Write as IoWrite};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::NodeServerError;

// ── InviteBundle ──────────────────────────────────────────────────────────────

/// The plaintext payload of a Node invitation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteBundle {
    /// Unique invite token ID — matches [`crate::invite::token::InviteToken::id`].
    pub token_id: Uuid,

    /// Public HTTPS URL of the inviting node (e.g. `"https://node.example.com"`).
    pub node_url: String,

    /// TCP port allocated for this invite's setup handshake.
    pub invite_port: u16,

    /// When this invite expires (UTC).
    pub expires_at: DateTime<Utc>,

    /// Human-readable label set by the admin (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

impl InviteBundle {
    /// Create a new bundle.
    #[must_use]
    pub fn new(
        token_id: Uuid,
        node_url: impl Into<String>,
        invite_port: u16,
        expires_at: DateTime<Utc>,
        label: Option<String>,
    ) -> Self {
        Self {
            token_id,
            node_url: node_url.into(),
            invite_port,
            expires_at,
            label,
        }
    }

    // ── Encryption ───────────────────────────────────────────────────────────

    /// Serialize and encrypt this bundle using `passphrase` (the raw invite token string).
    ///
    /// Returns armored age ciphertext (ASCII).
    ///
    /// # Errors
    ///
    /// Returns [`NodeServerError`] if TOML serialization or age encryption fails.
    pub fn encrypt(&self, passphrase: &str) -> Result<String, NodeServerError> {
        let plaintext = toml::to_string_pretty(self)
            .map_err(|e| NodeServerError::Invite(format!("TOML serialize: {e}")))?;

        let secret = age::secrecy::SecretString::from(passphrase.to_owned());
        let encryptor = age::Encryptor::with_user_passphrase(secret);

        let mut ciphertext: Vec<u8> = Vec::new();
        let armored =
            age::armor::ArmoredWriter::wrap_output(&mut ciphertext, age::armor::Format::AsciiArmor)
                .map_err(|e| NodeServerError::Invite(format!("armor init: {e}")))?;
        let mut writer = encryptor
            .wrap_output(armored)
            .map_err(|e| NodeServerError::Invite(format!("age wrap: {e}")))?;
        writer.write_all(plaintext.as_bytes())?;
        let armored = writer
            .finish()
            .map_err(|e| NodeServerError::Invite(format!("age finish: {e}")))?;
        armored
            .finish()
            .map_err(|e| NodeServerError::Invite(format!("armor finish: {e}")))?;

        String::from_utf8(ciphertext).map_err(|e| NodeServerError::Invite(format!("utf8: {e}")))
    }

    /// Decrypt and deserialize a bundle produced by [`InviteBundle::encrypt`].
    ///
    /// # Errors
    ///
    /// Returns [`NodeServerError`] if decryption or TOML parsing fails.
    pub fn decrypt(armored: &str, passphrase: &str) -> Result<Self, NodeServerError> {
        let secret = age::secrecy::SecretString::from(passphrase.to_owned());
        let identity = age::scrypt::Identity::new(secret);

        let reader = age::armor::ArmoredReader::new(armored.as_bytes());
        let decryptor = age::Decryptor::new(reader)
            .map_err(|e| NodeServerError::Invite(format!("age decryptor: {e}")))?;
        let mut plaintext = String::new();
        decryptor
            .decrypt(std::iter::once(&identity as &dyn age::Identity))
            .map_err(|e| NodeServerError::Invite(format!("age decrypt: {e}")))?
            .read_to_string(&mut plaintext)?;

        toml::from_str(&plaintext)
            .map_err(|e| NodeServerError::Invite(format!("TOML deserialize: {e}")))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_bundle() -> InviteBundle {
        InviteBundle::new(
            Uuid::new_v4(),
            "https://node.example.com",
            9150,
            Utc::now() + chrono::Duration::hours(24),
            Some("test-invite".into()),
        )
    }

    #[test]
    fn encrypt_decrypt_round_trip() {
        let bundle = sample_bundle();
        let passphrase = "super-secret-token-abc123";

        let encrypted = bundle.encrypt(passphrase).expect("encrypt must succeed");
        let decrypted =
            InviteBundle::decrypt(&encrypted, passphrase).expect("decrypt must succeed");

        assert_eq!(bundle.token_id, decrypted.token_id);
        assert_eq!(bundle.node_url, decrypted.node_url);
        assert_eq!(bundle.invite_port, decrypted.invite_port);
        assert_eq!(bundle.label, decrypted.label);
    }

    #[test]
    fn wrong_passphrase_fails() {
        let bundle = sample_bundle();
        let encrypted = bundle.encrypt("correct-passphrase").expect("encrypt");
        assert!(InviteBundle::decrypt(&encrypted, "wrong-passphrase").is_err());
    }
}
