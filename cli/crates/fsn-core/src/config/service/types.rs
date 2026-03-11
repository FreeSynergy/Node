// ServiceType enum — functional role classification for service modules.
//
// Separated from the class/meta structs so this enum (used everywhere for
// filtering and slot-matching) can be imported without pulling in the full
// service definition (ContainerDef, HealthCheck, etc.).

use serde::{Deserialize, Serialize};

// ── ServiceType ───────────────────────────────────────────────────────────────

/// The functional role of a service.
///
/// A service may declare **multiple types** — e.g. Zentinel is both `Proxy`
/// and `WebhosterSimple`; Keycloak is both `IamProvider` and `IamBroker`.
/// Types determine which project slots a service can fill and which
/// typed interfaces it exposes.
///
/// TOML accepts either a single string (legacy) or an array:
///   type   = "proxy"               # legacy / single
///   types  = ["proxy", "webhoster_simple"]   # multi-type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum ServiceType {
    // ── IAM ──────────────────────────────────────────────────────────────
    /// Identity provider: issues tokens, handles login (Kanidm, Keycloak, …)
    IamProvider,
    /// Identity broker: federates external identity providers (Keycloak, …)
    IamBroker,
    /// Legacy catch-all for any IAM service (mapped to IamProvider on read)
    Iam,

    // ── Proxy / Webhosting ────────────────────────────────────────────────
    /// Reverse proxy / ingress with TLS termination (Zentinel, Caddy, …)
    Proxy,
    /// Simple static or app webhosting (no PHP/FPM) (Zentinel, …)
    WebhosterSimple,

    // ── Communication ─────────────────────────────────────────────────────
    /// Mail server (Stalwart, …)
    Mail,
    /// Team chat / Matrix (Tuwunel, …)
    Chat,

    // ── Developer tools ───────────────────────────────────────────────────
    /// Git hosting (Forgejo, Gitea, …)
    Git,

    // ── Knowledge & collaboration ─────────────────────────────────────────
    /// Wiki / knowledge base (Outline, BookStack, …)
    Wiki,
    /// Collaborative editing (CryptPad, …)
    Collab,

    // ── Project management ────────────────────────────────────────────────
    /// Issue / task tracker (Vikunja, …)
    Tasks,
    /// Ticketing / event shop (Pretix, …)
    Tickets,

    // ── Geo & maps ────────────────────────────────────────────────────────
    /// Maps & geo (uMap, …)
    Maps,

    // ── Observability ─────────────────────────────────────────────────────
    /// Observability / metrics / logs (OpenObserver, …)
    Monitoring,

    // ── Infrastructure (internal) ─────────────────────────────────────────
    /// Relational database (Postgres) – internal, no proxy route
    Database,
    /// Key-value cache (Dragonfly/Redis) – internal, no proxy route
    Cache,

    // ── Bots / automation ─────────────────────────────────────────────────
    /// Bot / automation agent (Matrix bot, Telegram bot, …)
    Bot,

    /// User-defined / unknown type
    #[serde(other)]
    #[default]
    Custom,
}

impl std::fmt::Display for ServiceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ServiceType::IamProvider     => "iam_provider",
            ServiceType::IamBroker       => "iam_broker",
            ServiceType::Iam             => "iam",
            ServiceType::Proxy           => "proxy",
            ServiceType::WebhosterSimple => "webhoster_simple",
            ServiceType::Mail            => "mail",
            ServiceType::Chat            => "chat",
            ServiceType::Git             => "git",
            ServiceType::Wiki            => "wiki",
            ServiceType::Collab          => "collab",
            ServiceType::Tasks           => "tasks",
            ServiceType::Tickets         => "tickets",
            ServiceType::Maps            => "maps",
            ServiceType::Monitoring      => "monitoring",
            ServiceType::Database        => "database",
            ServiceType::Cache           => "cache",
            ServiceType::Bot             => "bot",
            ServiceType::Custom          => "custom",
        };
        write!(f, "{s}")
    }
}

impl ServiceType {
    /// Returns `true` for types that are internal infrastructure
    /// (no subdomain, no proxy route, no user-facing UI).
    pub fn is_internal(&self) -> bool {
        matches!(self, ServiceType::Database | ServiceType::Cache)
    }

    /// Returns `true` if this type can fill the IAM slot of a project.
    pub fn is_iam(&self) -> bool {
        matches!(self, ServiceType::IamProvider | ServiceType::IamBroker | ServiceType::Iam)
    }

    /// Returns `true` if this type can act as the project's reverse proxy.
    pub fn is_proxy(&self) -> bool {
        matches!(self, ServiceType::Proxy)
    }

    /// Logical category this type belongs to.
    ///
    /// Used for grouping in the service slot type-filter.
    /// Multiple types may share the same category (e.g. IamProvider + IamBroker → "iam").
    pub fn category(&self) -> &'static str {
        match self {
            ServiceType::IamProvider | ServiceType::IamBroker | ServiceType::Iam => "iam",
            ServiceType::Proxy | ServiceType::WebhosterSimple                    => "proxy",
            ServiceType::Mail  | ServiceType::Chat                               => "communication",
            ServiceType::Git                                                     => "developer",
            ServiceType::Wiki  | ServiceType::Collab                             => "knowledge",
            ServiceType::Tasks | ServiceType::Tickets                            => "project",
            ServiceType::Maps                                                    => "geo",
            ServiceType::Monitoring                                              => "monitoring",
            ServiceType::Database | ServiceType::Cache                          => "infrastructure",
            ServiceType::Bot                                                     => "automation",
            ServiceType::Custom                                                  => "custom",
        }
    }

    /// Human-readable label (English) for TUI display.
    pub fn label(&self) -> &'static str {
        match self {
            ServiceType::IamProvider     => "IAM Provider",
            ServiceType::IamBroker       => "IAM Broker",
            ServiceType::Iam             => "IAM",
            ServiceType::Proxy           => "Reverse Proxy",
            ServiceType::WebhosterSimple => "Webhoster (Simple)",
            ServiceType::Mail            => "Mail Server",
            ServiceType::Chat            => "Team Chat",
            ServiceType::Git             => "Git Hosting",
            ServiceType::Wiki            => "Wiki",
            ServiceType::Collab          => "Collaborative Editing",
            ServiceType::Tasks           => "Task Tracker",
            ServiceType::Tickets         => "Ticketing",
            ServiceType::Maps            => "Maps",
            ServiceType::Monitoring      => "Monitoring",
            ServiceType::Database        => "Database",
            ServiceType::Cache           => "Cache",
            ServiceType::Bot             => "Bot",
            ServiceType::Custom          => "Custom",
        }
    }
}

// ── Multi-type deserializer ────────────────────────────────────────────────────

/// Deserialize `service_types` from either a single string or an array.
///
/// This enables backward-compatible reading of legacy TOML files that used
/// `type = "proxy"` alongside new files that use `types = ["proxy", "webhoster_simple"]`.
pub fn de_service_types<'de, D>(d: D) -> Result<Vec<ServiceType>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::{self, SeqAccess, Visitor};
    use std::fmt;

    struct TypesVisitor;
    impl<'de> Visitor<'de> for TypesVisitor {
        type Value = Vec<ServiceType>;
        fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("a service type string or array of service type strings")
        }
        // Single string: `type = "proxy"`
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            let t: ServiceType = serde::Deserialize::deserialize(
                serde::de::value::StrDeserializer::new(v)
            )?;
            Ok(vec![t])
        }
        // Array: `types = ["proxy", "webhoster_simple"]`
        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut types = Vec::new();
            while let Some(t) = seq.next_element::<ServiceType>()? {
                types.push(t);
            }
            Ok(types)
        }
    }

    d.deserialize_any(TypesVisitor)
}
