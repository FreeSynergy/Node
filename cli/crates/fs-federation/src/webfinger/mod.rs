// fs-federation/src/webfinger/mod.rs — WebFinger (RFC 7033) client + types

use fs_error::FsError;
use serde::{Deserialize, Serialize};

// ── WebFingerLink ──────────────────────────────────────────────────────────────

/// A single link entry within a WebFinger JRD response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFingerLink {
    /// Link relation type (e.g. `"self"`, `"http://webfinger.net/rel/profile-page"`).
    pub rel: String,
    /// MIME type of the linked resource (optional).
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub link_type: Option<String>,
    /// Target URL of the link (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

// ── WebFingerResponse ──────────────────────────────────────────────────────────

/// WebFinger JRD (JSON Resource Descriptor) response (RFC 7033 §4.4).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebFingerResponse {
    /// The queried resource URI (e.g. `"acct:user@example.com"`).
    pub subject: String,
    /// Alternative URIs for the same resource.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aliases: Vec<String>,
    /// Resource links with relation types and target URLs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub links: Vec<WebFingerLink>,
}

impl WebFingerResponse {
    /// Find the first link matching the given `rel` value.
    pub fn link(&self, rel: &str) -> Option<&WebFingerLink> {
        self.links.iter().find(|l| l.rel == rel)
    }

    /// Return the `href` of the first `"self"` link, if present.
    pub fn self_href(&self) -> Option<&str> {
        self.link("self").and_then(|l| l.href.as_deref())
    }

    /// Return the `href` of the first profile page link, if present.
    pub fn profile_href(&self) -> Option<&str> {
        self.link("http://webfinger.net/rel/profile-page")
            .and_then(|l| l.href.as_deref())
    }
}

// ── WebFingerClient ────────────────────────────────────────────────────────────

/// WebFinger client for looking up resources on remote hosts (RFC 7033).
pub struct WebFingerClient {
    http: reqwest::Client,
}

impl WebFingerClient {
    /// Create a new [`WebFingerClient`].
    pub fn new() -> Self {
        Self {
            http: reqwest::Client::new(),
        }
    }

    /// Look up a `resource` URI (e.g. `"acct:alice@example.com"`) on `host`
    /// (e.g. `"example.com"`).
    ///
    /// Queries `https://{host}/.well-known/webfinger?resource={resource}`.
    pub async fn lookup(&self, host: &str, resource: &str) -> Result<WebFingerResponse, FsError> {
        let url = format!(
            "https://{}/.well-known/webfinger?resource={}",
            host,
            urlencoding::encode(resource)
        );

        let resp = self
            .http
            .get(&url)
            .header("Accept", "application/jrd+json, application/json")
            .send()
            .await
            .map_err(|e| FsError::network(format!("WebFinger request failed: {e}")))?;

        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(FsError::not_found(format!(
                "WebFinger resource not found: {resource}"
            )));
        }
        if !resp.status().is_success() {
            return Err(FsError::network(format!(
                "WebFinger returned HTTP {}",
                resp.status()
            )));
        }

        resp.json::<WebFingerResponse>()
            .await
            .map_err(|e| FsError::parse(format!("WebFinger JSON parse failed: {e}")))
    }

    /// Convenience helper: look up `acct:{user}@{host}` on `host`.
    pub async fn lookup_acct(&self, user: &str, host: &str) -> Result<WebFingerResponse, FsError> {
        let resource = format!("acct:{user}@{host}");
        self.lookup(host, &resource).await
    }
}

impl Default for WebFingerClient {
    fn default() -> Self {
        Self::new()
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_lookup() {
        let resp = WebFingerResponse {
            subject: "acct:alice@example.com".into(),
            aliases: vec![],
            links: vec![
                WebFingerLink {
                    rel: "self".into(),
                    link_type: Some("application/activity+json".into()),
                    href: Some("https://example.com/users/alice".into()),
                },
                WebFingerLink {
                    rel: "http://webfinger.net/rel/profile-page".into(),
                    link_type: None,
                    href: Some("https://example.com/@alice".into()),
                },
            ],
        };

        assert_eq!(resp.self_href(), Some("https://example.com/users/alice"));
        assert_eq!(resp.profile_href(), Some("https://example.com/@alice"));
        assert!(resp
            .link("http://ostatus.org/schema/1.0/subscribe")
            .is_none());
    }

    #[test]
    fn empty_links_serializes_clean() {
        let resp = WebFingerResponse {
            subject: "acct:bob@example.com".into(),
            aliases: vec![],
            links: vec![],
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(!json.contains("links"));
        assert!(!json.contains("aliases"));
    }
}
