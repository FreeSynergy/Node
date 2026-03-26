// fs-auth/src/jwt.rs — JWT signing and validation (feature = "jwt")

use fs_error::FsError;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::claims::Claims;

// ── JwtAlgorithm ──────────────────────────────────────────────────────────────

/// Algorithm used for JWT signing and verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JwtAlgorithm {
    /// HMAC-SHA256 (symmetric).
    HS256,
    /// RSA-SHA256 (asymmetric).
    RS256,
}

impl From<JwtAlgorithm> for Algorithm {
    fn from(alg: JwtAlgorithm) -> Self {
        match alg {
            JwtAlgorithm::HS256 => Algorithm::HS256,
            JwtAlgorithm::RS256 => Algorithm::RS256,
        }
    }
}

// ── JwtSigner ─────────────────────────────────────────────────────────────────

/// Signs [`Claims`] into JWT strings.
pub struct JwtSigner {
    key: EncodingKey,
    algorithm: JwtAlgorithm,
    issuer: String,
    audience: String,
}

impl JwtSigner {
    /// Create a signer that uses HMAC-SHA256 with the given `secret`.
    pub fn hmac(secret: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            key: EncodingKey::from_secret(secret),
            algorithm: JwtAlgorithm::HS256,
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    /// Create a signer that uses RSA-SHA256 with the given PEM-encoded private key.
    pub fn rsa_pem(
        pem: &[u8],
        issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self, FsError> {
        let key = EncodingKey::from_rsa_pem(pem)
            .map_err(|e| FsError::auth(format!("invalid RSA private key PEM: {e}")))?;
        Ok(Self {
            key,
            algorithm: JwtAlgorithm::RS256,
            issuer: issuer.into(),
            audience: audience.into(),
        })
    }

    /// Return the configured issuer for this signer.
    pub fn issuer(&self) -> &str {
        &self.issuer
    }

    /// Return the configured audience for this signer.
    pub fn audience(&self) -> &str {
        &self.audience
    }

    /// Sign `claims` and return the JWT string.
    ///
    /// Returns an error if `claims.iss` or `claims.aud` do not match the signer's
    /// configured issuer and audience.
    pub fn sign(&self, claims: &Claims) -> Result<String, FsError> {
        if claims.iss != self.issuer {
            return Err(FsError::auth(format!(
                "JWT issuer mismatch: claims.iss={:?}, signer.issuer={:?}",
                claims.iss, self.issuer
            )));
        }
        if claims.aud != self.audience {
            return Err(FsError::auth(format!(
                "JWT audience mismatch: claims.aud={:?}, signer.audience={:?}",
                claims.aud, self.audience
            )));
        }
        let header = Header::new(self.algorithm.into());
        encode(&header, claims, &self.key)
            .map_err(|e| FsError::auth(format!("JWT signing failed: {e}")))
    }
}

// ── JwtValidator ──────────────────────────────────────────────────────────────

/// Validates JWT strings and decodes them into [`Claims`].
pub struct JwtValidator {
    key: DecodingKey,
    algorithm: JwtAlgorithm,
    issuer: String,
    audience: String,
}

impl JwtValidator {
    /// Create a validator that uses HMAC-SHA256 with the given `secret`.
    pub fn hmac(secret: &[u8], issuer: impl Into<String>, audience: impl Into<String>) -> Self {
        Self {
            key: DecodingKey::from_secret(secret),
            algorithm: JwtAlgorithm::HS256,
            issuer: issuer.into(),
            audience: audience.into(),
        }
    }

    /// Create a validator that uses RSA-SHA256 with the given PEM-encoded public key.
    pub fn rsa_pem(
        pem: &[u8],
        issuer: impl Into<String>,
        audience: impl Into<String>,
    ) -> Result<Self, FsError> {
        let key = DecodingKey::from_rsa_pem(pem)
            .map_err(|e| FsError::auth(format!("invalid RSA public key PEM: {e}")))?;
        Ok(Self {
            key,
            algorithm: JwtAlgorithm::RS256,
            issuer: issuer.into(),
            audience: audience.into(),
        })
    }

    /// Validate and decode `token`, returning [`Claims`] on success.
    ///
    /// Verifies signature, expiry, issuer, and audience.
    pub fn validate(&self, token: &str) -> Result<Claims, FsError> {
        let mut validation = Validation::new(self.algorithm.into());
        validation.set_issuer(&[&self.issuer]);
        validation.set_audience(&[&self.audience]);

        decode::<Claims>(token, &self.key, &validation)
            .map(|data| data.claims)
            .map_err(|e| FsError::auth(format!("JWT validation failed: {e}")))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::permission::PermissionSet;

    #[test]
    fn hmac_round_trip() {
        let secret = b"super-secret-key";
        let signer = JwtSigner::hmac(secret, "fsn", "api");
        let validator = JwtValidator::hmac(secret, "fsn", "api");

        let claims = Claims::new("user:1", "fsn", "api", 3600, PermissionSet::default());
        let token = signer.sign(&claims).expect("sign");
        let decoded = validator.validate(&token).expect("validate");
        assert_eq!(decoded.sub, "user:1");
    }
}
