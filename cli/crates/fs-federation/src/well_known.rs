// well_known.rs — `.well-known` discovery endpoint helpers.
//
// Provides URL builders and response types for the standard discovery endpoints
// that federated services expose:
//
//   /.well-known/openid-configuration   (OIDC discovery — RFC 8414)
//   /.well-known/webfinger              (WebFinger — RFC 7033)
//   /.well-known/nodeinfo               (NodeInfo — ActivityPub nodes)
//   /.well-known/host-meta              (host-meta — RFC 6415)
//
// These are passive helpers — they produce the URLs and deserialise the
// responses; they do NOT serve the endpoints themselves (that is the job of
// the running service such as Kanidm or Tuwunel).

use serde::{Deserialize, Serialize};

// ── WellKnownEndpoint ─────────────────────────────────────────────────────────

/// Standard `.well-known` path constants.
pub struct WellKnownPath;

impl WellKnownPath {
    /// OIDC / OAuth 2.0 server metadata endpoint (RFC 8414).
    pub const OPENID_CONFIGURATION: &'static str = "/.well-known/openid-configuration";

    /// WebFinger resource-discovery endpoint (RFC 7033).
    pub const WEBFINGER: &'static str = "/.well-known/webfinger";

    /// NodeInfo discovery pointer for ActivityPub nodes.
    pub const NODEINFO: &'static str = "/.well-known/nodeinfo";

    /// host-meta document (RFC 6415 / XRD).
    pub const HOST_META: &'static str = "/.well-known/host-meta";

    /// Build the absolute HTTPS URL for the given `path` on `host`.
    ///
    /// ```
    /// use fs_federation::well_known::WellKnownPath;
    ///
    /// assert_eq!(
    ///     WellKnownPath::url("auth.example.com", WellKnownPath::OPENID_CONFIGURATION),
    ///     "https://auth.example.com/.well-known/openid-configuration",
    /// );
    /// ```
    pub fn url(host: &str, path: &str) -> String {
        let host = host.trim_end_matches('/');
        let path = if path.starts_with('/') {
            path
        } else {
            &format!("/{path}")
        };
        format!("https://{host}{path}")
    }

    /// Build the OIDC discovery URL for `host`.
    pub fn oidc_configuration(host: &str) -> String {
        Self::url(host, Self::OPENID_CONFIGURATION)
    }

    /// Build the WebFinger URL for `host` with optional `resource` query param.
    pub fn webfinger(host: &str, resource: Option<&str>) -> String {
        let base = Self::url(host, Self::WEBFINGER);
        match resource {
            Some(r) => format!("{}?resource={}", base, urlencoding::encode(r)),
            None => base,
        }
    }

    /// Build the NodeInfo discovery URL for `host`.
    pub fn nodeinfo(host: &str) -> String {
        Self::url(host, Self::NODEINFO)
    }
}

// ── NodeInfoPointer ───────────────────────────────────────────────────────────

/// `/.well-known/nodeinfo` response — pointer to the full NodeInfo document.
///
/// <https://nodeinfo.diaspora.software/protocol.html>
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoPointer {
    /// List of links pointing to versioned NodeInfo documents.
    pub links: Vec<NodeInfoLink>,
}

/// A single link in the `/.well-known/nodeinfo` response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfoLink {
    /// NodeInfo schema URL, e.g.
    /// `"http://nodeinfo.diaspora.software/ns/schema/2.1"`.
    pub rel: String,

    /// Absolute URL of the full NodeInfo document.
    pub href: String,
}

impl NodeInfoPointer {
    /// Return the href for the highest supported NodeInfo version, if present.
    ///
    /// Prefers 2.1 over 2.0 over any other version.
    pub fn best_href(&self) -> Option<&str> {
        const ORDER: &[&str] = &[
            "http://nodeinfo.diaspora.software/ns/schema/2.1",
            "http://nodeinfo.diaspora.software/ns/schema/2.0",
        ];
        for schema in ORDER {
            if let Some(link) = self.links.iter().find(|l| l.rel.as_str() == *schema) {
                return Some(&link.href);
            }
        }
        // Fallback: first available
        self.links.first().map(|l| l.href.as_str())
    }
}

// ── HostMeta ──────────────────────────────────────────────────────────────────

/// Minimal `/.well-known/host-meta` JSON representation (RFC 6415).
///
/// Many federated services (e.g. Mastodon, Matrix) use this to advertise
/// their WebFinger endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMeta {
    /// List of link-relation entries.
    #[serde(default)]
    pub links: Vec<HostMetaLink>,
}

/// A single link in the host-meta document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMetaLink {
    /// Link relation type.
    pub rel: String,
    /// URL template or target URL.
    pub template: Option<String>,
    /// Optional direct href.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}

impl HostMeta {
    /// Build a standard host-meta for a domain that uses WebFinger.
    ///
    /// Produces a single lrdd link pointing to the WebFinger template.
    ///
    /// ```
    /// use fs_federation::well_known::HostMeta;
    ///
    /// let hm = HostMeta::for_webfinger("example.com");
    /// let link = &hm.links[0];
    /// assert_eq!(link.rel, "lrdd");
    /// assert!(link.template.as_deref().unwrap().contains("example.com"));
    /// ```
    pub fn for_webfinger(host: &str) -> Self {
        let template = format!(
            "https://{}/.well-known/webfinger?resource={{{{uri}}}}",
            host.trim_end_matches('/')
        );
        Self {
            links: vec![HostMetaLink {
                rel: "lrdd".into(),
                template: Some(template),
                href: None,
            }],
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_builder_oidc() {
        assert_eq!(
            WellKnownPath::oidc_configuration("auth.example.com"),
            "https://auth.example.com/.well-known/openid-configuration",
        );
    }

    #[test]
    fn url_builder_webfinger_with_resource() {
        let url = WellKnownPath::webfinger("example.com", Some("acct:alice@example.com"));
        assert!(url.starts_with("https://example.com/.well-known/webfinger?resource="));
        assert!(url.contains("alice"));
    }

    #[test]
    fn url_builder_webfinger_without_resource() {
        assert_eq!(
            WellKnownPath::webfinger("example.com", None),
            "https://example.com/.well-known/webfinger",
        );
    }

    #[test]
    fn url_builder_strips_trailing_slash() {
        let url = WellKnownPath::url("example.com/", "/.well-known/nodeinfo");
        assert_eq!(url, "https://example.com/.well-known/nodeinfo");
    }

    #[test]
    fn nodeinfo_pointer_best_href_prefers_2_1() {
        let pointer = NodeInfoPointer {
            links: vec![
                NodeInfoLink {
                    rel: "http://nodeinfo.diaspora.software/ns/schema/2.0".into(),
                    href: "https://example.com/nodeinfo/2.0".into(),
                },
                NodeInfoLink {
                    rel: "http://nodeinfo.diaspora.software/ns/schema/2.1".into(),
                    href: "https://example.com/nodeinfo/2.1".into(),
                },
            ],
        };
        assert_eq!(
            pointer.best_href(),
            Some("https://example.com/nodeinfo/2.1"),
        );
    }

    #[test]
    fn nodeinfo_pointer_fallback_first() {
        let pointer = NodeInfoPointer {
            links: vec![NodeInfoLink {
                rel: "http://nodeinfo.diaspora.software/ns/schema/1.0".into(),
                href: "https://example.com/nodeinfo/1.0".into(),
            }],
        };
        assert_eq!(
            pointer.best_href(),
            Some("https://example.com/nodeinfo/1.0"),
        );
    }

    #[test]
    fn nodeinfo_pointer_empty_returns_none() {
        let pointer = NodeInfoPointer { links: vec![] };
        assert!(pointer.best_href().is_none());
    }

    #[test]
    fn host_meta_for_webfinger() {
        let hm = HostMeta::for_webfinger("example.com");
        assert_eq!(hm.links.len(), 1);
        let link = &hm.links[0];
        assert_eq!(link.rel, "lrdd");
        assert!(link.template.as_deref().unwrap().contains("example.com"));
        assert!(link
            .template
            .as_deref()
            .unwrap()
            .contains(".well-known/webfinger"));
    }

    #[test]
    fn host_meta_serializes_without_null_href() {
        let hm = HostMeta::for_webfinger("example.com");
        let json = serde_json::to_string(&hm).unwrap();
        assert!(!json.contains("\"href\":null"));
    }
}
