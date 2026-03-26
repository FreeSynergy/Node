// fs-federation/src/scim/mod.rs — SCIM 2.0 user/group types + provisioning client

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── ScimEmail ─────────────────────────────────────────────────────────────────

/// An email address entry within a SCIM user resource.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    /// The email address.
    pub value: String,
    /// `true` when this is the user's primary email.
    pub primary: bool,
}

// ── ScimUser ──────────────────────────────────────────────────────────────────

/// SCIM 2.0 user resource (RFC 7643 §4.1).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUser {
    /// Server-assigned unique identifier.
    pub id: Option<String>,
    /// Unique username within the provider.
    #[serde(rename = "userName")]
    pub user_name: String,
    /// Human-readable display name.
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    /// List of email addresses for this user.
    pub emails: Vec<ScimEmail>,
    /// Whether the user account is active.
    pub active: bool,
}

// ── ScimMember ────────────────────────────────────────────────────────────────

/// A member reference within a SCIM group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMember {
    /// The referenced user's ID.
    pub value: String,
    /// Human-readable display name of the referenced user.
    pub display: Option<String>,
}

// ── ScimGroup ─────────────────────────────────────────────────────────────────

/// SCIM 2.0 group resource (RFC 7643 §4.2).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroup {
    /// Server-assigned unique identifier.
    pub id: Option<String>,
    /// Human-readable group name.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// Members belonging to this group.
    pub members: Vec<ScimMember>,
}

// ── Private helpers ───────────────────────────────────────────────────────────

/// Internal deserialization wrapper for SCIM list responses.
#[derive(Deserialize)]
struct ScimListResponse<T> {
    #[serde(rename = "Resources")]
    resources: Vec<T>,
    #[allow(dead_code)]
    #[serde(rename = "totalResults")]
    total_results: u64,
}

// ── ScimClient ────────────────────────────────────────────────────────────────

/// SCIM 2.0 provisioning client.
pub struct ScimClient {
    base_url: String,
    bearer_token: String,
    http: reqwest::Client,
}

impl ScimClient {
    /// Create a new [`ScimClient`].
    ///
    /// `base_url` should be the SCIM base endpoint (e.g. `"https://idp.example.com/scim/v2"`).
    pub fn new(base_url: impl Into<String>, bearer_token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            bearer_token: bearer_token.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Create a new user via SCIM `POST /Users`.
    pub async fn create_user(&self, user: &ScimUser) -> Result<ScimUser, FsError> {
        let url = format!("{}/Users", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.bearer_token)
            .json(user)
            .send()
            .await
            .map_err(|e| FsError::network(format!("SCIM create_user request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(FsError::internal(format!(
                "SCIM create_user returned HTTP {}",
                resp.status()
            )));
        }

        resp.json::<ScimUser>()
            .await
            .map_err(|e| FsError::parse(format!("SCIM create_user JSON parse failed: {e}")))
    }

    /// Fetch a single user by ID via SCIM `GET /Users/{id}`.
    pub async fn get_user(&self, id: &str) -> Result<ScimUser, FsError> {
        let url = format!("{}/Users/{}", self.base_url, id);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await
            .map_err(|e| FsError::network(format!("SCIM get_user request failed: {e}")))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(FsError::not_found(format!("SCIM user not found: {id}")));
        }
        if !resp.status().is_success() {
            return Err(FsError::internal(format!(
                "SCIM get_user returned HTTP {}",
                resp.status()
            )));
        }

        resp.json::<ScimUser>()
            .await
            .map_err(|e| FsError::parse(format!("SCIM get_user JSON parse failed: {e}")))
    }

    /// List all users via SCIM `GET /Users`.
    pub async fn list_users(&self) -> Result<Vec<ScimUser>, FsError> {
        let url = format!("{}/Users", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await
            .map_err(|e| FsError::network(format!("SCIM list_users request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(FsError::internal(format!(
                "SCIM list_users returned HTTP {}",
                resp.status()
            )));
        }

        let list = resp
            .json::<ScimListResponse<ScimUser>>()
            .await
            .map_err(|e| FsError::parse(format!("SCIM list_users JSON parse failed: {e}")))?;
        Ok(list.resources)
    }

    /// Create a new group via SCIM `POST /Groups`.
    pub async fn create_group(&self, group: &ScimGroup) -> Result<ScimGroup, FsError> {
        let url = format!("{}/Groups", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.bearer_token)
            .json(group)
            .send()
            .await
            .map_err(|e| FsError::network(format!("SCIM create_group request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(FsError::internal(format!(
                "SCIM create_group returned HTTP {}",
                resp.status()
            )));
        }

        resp.json::<ScimGroup>()
            .await
            .map_err(|e| FsError::parse(format!("SCIM create_group JSON parse failed: {e}")))
    }

    /// List all groups via SCIM `GET /Groups`.
    pub async fn list_groups(&self) -> Result<Vec<ScimGroup>, FsError> {
        let url = format!("{}/Groups", self.base_url);
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.bearer_token)
            .send()
            .await
            .map_err(|e| FsError::network(format!("SCIM list_groups request failed: {e}")))?;

        if !resp.status().is_success() {
            return Err(FsError::internal(format!(
                "SCIM list_groups returned HTTP {}",
                resp.status()
            )));
        }

        let list = resp
            .json::<ScimListResponse<ScimGroup>>()
            .await
            .map_err(|e| FsError::parse(format!("SCIM list_groups JSON parse failed: {e}")))?;
        Ok(list.resources)
    }
}
