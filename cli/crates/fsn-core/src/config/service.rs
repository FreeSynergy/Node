// Service class definition – maps to modules/{type}/{name}/{name}.toml
//
// Field order (MANDATORY per RULES.md):
//   module → vars → load → container → environment → setup
//
// The TOML key `[module]` is kept for file-level compatibility;
// internally we use `ServiceMeta` / `ServiceClass`.

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use toml::Value;

use crate::config::manifest::ModuleManifest;
use crate::error::FsnError;
use crate::resource::Resource;

// ── Service Type ──────────────────────────────────────────────────────────────

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

// ── Service Class ─────────────────────────────────────────────────────────────

/// A service class definition (the template/blueprint for a service).
/// Loaded from modules/{type}/{name}/{name}.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClass {
    /// Metadata block – TOML key is `[module]` for file compatibility.
    #[serde(rename = "module")]
    pub meta: ServiceMeta,

    #[serde(default)]
    pub vars: IndexMap<String, Value>,

    #[serde(default)]
    pub load: ServiceLoad,

    pub container: ContainerDef,

    #[serde(default)]
    pub environment: IndexMap<String, String>,

    /// Setup wizard configuration – what this service needs before it can run.
    #[serde(default)]
    pub setup: ServiceSetup,

    /// Routing contract – what the service exposes to the proxy.
    /// Proxy modules iterate over all contracts to generate routing config.
    #[serde(default)]
    pub contract: ServiceContract,

    /// Plugin manifest – commands, inputs and outputs for the process plugin protocol.
    /// Absent for modules that have not yet been migrated to the plugin system.
    #[serde(default, rename = "plugin")]
    pub manifest: Option<ModuleManifest>,
}

// ── Setup wizard types ────────────────────────────────────────────────────────

// ── Service Contract ──────────────────────────────────────────────────────────

/// Routing and capability contract declared by a service module.
///
/// The proxy driver reads `ServiceContract` to generate per-service routing
/// config — analogous to a Kubernetes `Ingress` spec.  The service declares
/// what it needs; the proxy decides how to implement it.
///
/// Empty `routes` = no proxy routing generated (internal services).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceContract {
    /// HTTP routes this service exposes. Empty = proxy skips this service.
    #[serde(default)]
    pub routes: Vec<RouteSpec>,

    /// Extra HTTP headers the proxy injects when forwarding to this service.
    #[serde(default)]
    pub headers: Vec<HeaderSpec>,

    /// Whether the container speaks TLS internally.
    /// `true` → proxy uses HTTPS to reach the container (e.g. Kanidm).
    /// `false` (default) → proxy speaks plain HTTP to the container.
    #[serde(default)]
    pub upstream_tls: bool,

    /// Override the proxy health-check path for this service.
    /// Falls back to `module.health_path` when absent.
    pub health_path: Option<String>,
}

/// A URL route this service exposes through the proxy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteSpec {
    /// Unique identifier within this module (e.g. "main", "admin", "api").
    pub id: String,

    /// URL path prefix to match (e.g. "/" or "/auth").
    pub path: String,

    /// Strip the matched prefix before forwarding to the upstream.
    #[serde(default)]
    pub strip: bool,

    /// Human-readable description (shown in TUI and generated docs).
    pub description: Option<String>,
}

/// An HTTP header the proxy injects when forwarding requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderSpec {
    /// Header name (e.g. "X-Forwarded-Proto").
    pub name: String,
    /// Header value — Jinja2 templates allowed (e.g. "{{ service_domain }}").
    pub value: String,
}

/// All configuration fields this service requires during `fsn init`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceSetup {
    #[serde(default)]
    pub fields: Vec<SetupField>,
}

/// A single field the wizard will prompt for.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupField {
    /// Key to set: "vault_*" → stored in vault, anything else → env reminder.
    pub key: String,

    /// English label shown in prompt AND used as .po lookup key.
    pub label: String,

    /// Optional longer explanation shown below the prompt.
    pub description: Option<String>,

    #[serde(default)]
    pub field_type: FieldType,

    /// Auto-generate a random value; user can press Enter to accept or type override.
    #[serde(default)]
    pub auto_generate: bool,

    /// Pre-filled default value shown in the prompt.
    pub default: Option<String>,

    /// For FieldType::Select – the available choices.
    #[serde(default)]
    pub options: Vec<String>,

    /// Skip this field if the key already exists in vault (idempotent).
    #[serde(default = "default_true")]
    pub skip_if_set: bool,
}

fn default_true() -> bool { true }

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    #[default]
    String,
    Secret,  // masked input, stored in vault
    Email,
    Ip,
    Select,  // requires `options`
    Bool,
}

// ── Service Metadata ──────────────────────────────────────────────────────────

/// Core metadata declared under the `[module]` TOML key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceMeta {
    pub name: String,

    #[serde(default)]
    pub alias: Vec<String>,

    /// Functional types – determines typed interfaces and project slots.
    ///
    /// Accepts either `type = "proxy"` (legacy single string) or
    /// `types = ["proxy", "webhoster_simple"]` (multi-type array).
    /// Both keys are accepted; `types` takes precedence if both are present.
    #[serde(
        rename = "types",
        alias = "type",
        default,
        deserialize_with = "de_service_types"
    )]
    pub service_types: Vec<ServiceType>,

    pub author: Option<String>,
    pub version: String,

    #[serde(default)]
    pub tags: Vec<String>,

    pub description: Option<String>,
    pub website: Option<String>,
    pub repository: Option<String>,

    /// Primary internal port the service listens on.
    pub port: u16,

    #[serde(default)]
    pub constraints: Constraints,

    pub federation: Option<FederationMeta>,

    /// Path used by Zentinel upstream health checks.
    pub health_path: Option<String>,
    pub health_port: Option<u16>,
    pub health_scheme: Option<String>,
}

impl ServiceMeta {
    /// Returns `true` if this service is purely internal infrastructure
    /// (no subdomain, no proxy route, no user-facing UI).
    /// Requires ALL declared types to be internal.
    pub fn is_internal_only(&self) -> bool {
        !self.service_types.is_empty()
            && self.service_types.iter().all(|t| t.is_internal())
    }

    /// Returns `true` if any of the declared types matches `t`.
    pub fn has_type(&self, t: &ServiceType) -> bool {
        self.service_types.contains(t)
    }

    /// The primary type (first in the list), or `Custom` if the list is empty.
    pub fn primary_type(&self) -> &ServiceType {
        self.service_types.first().unwrap_or(&ServiceType::Custom)
    }

    /// Comma-separated label list for TUI display (e.g. "Reverse Proxy, Webhoster (Simple)").
    pub fn types_label(&self) -> String {
        if self.service_types.is_empty() {
            return ServiceType::Custom.label().to_string();
        }
        self.service_types.iter().map(|t| t.label()).collect::<Vec<_>>().join(", ")
    }
}

/// Deployment constraints declared per service class.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Constraints {
    /// Maximum number of instances of this service class per host (null = unlimited).
    pub per_host: Option<u32>,

    /// Maximum number of instances of this service class per IP (null = unlimited).
    pub per_ip: Option<u32>,

    /// Locality constraint – if Some(SameHost), must run on same host as consumer.
    pub locality: Option<Locality>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Locality {
    SameHost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationMeta {
    pub enabled: bool,
    pub min_trust: u8,
}

// ── Load / Dependencies ───────────────────────────────────────────────────────

/// Sub-service and service references declared under `[load]`.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceLoad {
    /// Sub-services this service owns and creates (e.g. postgres, dragonfly).
    /// TOML key: `modules` kept for file compatibility.
    #[serde(default, alias = "modules")]
    pub sub_services: IndexMap<String, SubServiceRef>,

    /// Other services whose config this service reads (no ownership).
    #[serde(default)]
    pub services: IndexMap<String, ServiceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubServiceRef {
    /// Class key, e.g. "database/postgres".
    /// TOML: `module_class` or `service_class` (both accepted).
    #[serde(alias = "module_class")]
    pub service_class: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceRef {}

// ── Container Definition ──────────────────────────────────────────────────────

/// Container definition – maps to the `[container]` TOML block.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerDef {
    pub name: String,
    pub image: String,
    pub image_tag: String,

    /// Auto-generated by engine – NEVER set manually in service TOML.
    #[serde(default)]
    pub networks: Vec<String>,

    #[serde(default)]
    pub volumes: Vec<String>,

    /// Forbidden on all services except proxy/zentinel.
    #[serde(default)]
    pub published_ports: Vec<String>,

    pub healthcheck: Option<HealthCheck>,

    /// Run as a specific UID[:GID] (e.g. "1000" or "15371:15371").
    pub user: Option<String>,

    #[serde(default)]
    pub read_only: bool,

    #[serde(default)]
    pub tmpfs: Vec<String>,

    #[serde(default)]
    pub security_opt: Vec<String>,

    #[serde(default)]
    pub ulimits: IndexMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub cmd: String,
    pub interval: String,
    pub timeout: String,
    pub retries: u32,
    pub start_period: String,
}

// ── Resource impl for ServiceClass ────────────────────────────────────────────

impl Resource for ServiceClass {
    fn kind(&self) -> &'static str { "service_class" }
    fn id(&self) -> &str { &self.meta.name }
    fn display_name(&self) -> &str { &self.meta.name }
    fn description(&self) -> Option<&str> { self.meta.description.as_deref() }
    fn tags(&self) -> &[String] { &self.meta.tags }

    fn validate(&self) -> Result<(), FsnError> {
        if self.meta.name.is_empty() {
            return Err(FsnError::ConstraintViolation { message: "module.name is required".into() });
        }
        if self.meta.version.is_empty() {
            return Err(FsnError::ConstraintViolation { message: "module.version is required".into() });
        }
        if self.container.image.is_empty() {
            return Err(FsnError::ConstraintViolation { message: "container.image is required".into() });
        }
        Ok(())
    }
}
