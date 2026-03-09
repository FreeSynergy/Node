// Module plugin manifest — the [plugin] block inside service module TOML files.
//
// Design: Process Protocol (stdin/stdout JSON).
// Core spawns the plugin executable, writes a PluginContext to stdin,
// reads a PluginResponse from stdout, then executes the instructions:
//   - writes declared files to disk
//   - runs declared shell commands (as unprivileged user)
//
// Protocol version 1 is the initial stable contract.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// ── Module manifest (declared in TOML) ────────────────────────────────────────

/// The `[plugin]` block in a service module TOML.
///
/// Declares what commands the module supports, what cross-service data it
/// needs from Core, and what files it produces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModuleManifest {
    /// Commands this module handles (e.g. ["deploy", "clean", "generate-config"]).
    #[serde(default)]
    pub commands: Vec<String>,

    /// Cross-service data the module needs — Core collects and injects these.
    #[serde(default)]
    pub inputs: ManifestInputs,

    /// Files this module generates (Core writes them after plugin runs).
    #[serde(default, rename = "outputs")]
    pub output_files: Vec<ManifestOutputFile>,

    /// Passive/data-only plugin: no container, no deploy, just exposes config data.
    #[serde(default)]
    pub external: bool,

    /// JSON protocol version — must be 1.
    #[serde(default = "protocol_v1")]
    pub protocol: u32,
}

fn protocol_v1() -> u32 { 1 }

/// Cross-service inputs a module may request.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ManifestInputs {
    /// Receive the full list of peer services (their domains, ports, types).
    /// Used by proxy, mail, wiki, IAM to produce per-service routing configs.
    #[serde(default)]
    pub services: bool,

    /// Receive IAM service vars (IAM_URL, IAM_DOMAIN, …) when an IAM service exists.
    #[serde(default)]
    pub iam_vars: bool,
}

/// A file the module plugin generates.
///
/// Core renders the template with Jinja2 and writes the result to `dest`.
/// The template path is relative to the module's Store directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestOutputFile {
    /// Identifier used in logs and dependency tracking (e.g. "proxy-config").
    pub name: String,

    /// Template file path, relative to the module's Store directory.
    /// E.g. "templates/zentinel.kdl.j2"
    pub template: String,

    /// Absolute destination path — Jinja2 vars like `{{ data_root }}` are expanded.
    pub dest: String,
}

// ── Plugin context (Core → Plugin via stdin) ──────────────────────────────────

/// JSON payload Core writes to a plugin's stdin.
///
/// The plugin reads this, processes the requested command, and writes a
/// `PluginResponse` to stdout.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginContext {
    /// Protocol version — plugin must reject if it doesn't support this.
    pub protocol: u32,

    /// The command to execute (must be in ModuleManifest::commands).
    pub command: String,

    /// The service instance being operated on.
    pub instance: InstanceInfo,

    /// Peer services in the same project (provided when ManifestInputs::services = true).
    #[serde(default)]
    pub peers: Vec<PeerService>,

    /// Cross-service environment variables collected from all peer services.
    /// E.g. IAM_URL, MAIL_DOMAIN, GIT_HOST, …
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// Information about the service instance being operated on.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    /// Instance name (e.g. "zentinel").
    pub name: String,

    /// Service class key (e.g. "proxy/zentinel").
    pub class_key: String,

    /// Fully qualified service domain (e.g. "zentinel.example.com").
    pub domain: String,

    /// Project slug.
    pub project: String,

    /// Project domain (e.g. "example.com").
    pub project_domain: String,

    /// Data root directory for this instance.
    pub data_root: String,

    /// Resolved environment variables for this instance (from [environment] + vault).
    #[serde(default)]
    pub env: HashMap<String, String>,
}

/// A peer service — one of the other services running in the same project.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerService {
    /// Instance name (e.g. "forgejo").
    pub name: String,

    /// Service class key (e.g. "git/forgejo").
    pub class_key: String,

    /// Functional types (e.g. ["git"]).
    pub types: Vec<String>,

    /// Fully qualified domain (e.g. "forgejo.example.com").
    pub domain: String,

    /// Primary port.
    pub port: u16,

    /// Whether the upstream speaks TLS internally.
    pub upstream_tls: bool,

    /// HTTP routes declared in the module's [contract].
    #[serde(default)]
    pub routes: Vec<PeerRoute>,

    /// Resolved environment vars exported by this peer (for cross-service injection).
    #[serde(default)]
    pub exported_vars: HashMap<String, String>,
}

/// A route declared by a peer service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerRoute {
    pub id: String,
    pub path: String,
    pub strip: bool,
}

// ── Plugin response (Plugin → Core via stdout) ────────────────────────────────

/// JSON payload the plugin writes to its stdout.
///
/// Core reads this, writes declared files, and runs declared shell commands
/// in order.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PluginResponse {
    /// Protocol version — Core rejects if it doesn't match.
    #[serde(default = "protocol_v1")]
    pub protocol: u32,

    /// Log lines to emit (Core prefixes with the module name).
    #[serde(default)]
    pub logs: Vec<LogLine>,

    /// Files to write to disk.
    #[serde(default)]
    pub files: Vec<OutputFile>,

    /// Shell commands to run after files are written (in order, must succeed).
    #[serde(default)]
    pub commands: Vec<ShellCommand>,

    /// Non-empty = plugin reported an error; Core aborts with this message.
    #[serde(default)]
    pub error: String,
}

/// A log line emitted by the plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogLine {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Info,
    Warn,
    Error,
}

/// A file the plugin wants Core to write.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputFile {
    /// Absolute destination path.
    pub dest: String,

    /// File contents (UTF-8 text).
    pub content: String,

    /// Unix permission bits (e.g. 0o644). Defaults to 0o644 if absent.
    #[serde(default = "default_mode")]
    pub mode: u32,
}

fn default_mode() -> u32 { 0o644 }

/// A shell command the plugin wants Core to run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShellCommand {
    /// The command and arguments (passed to sh -c).
    pub cmd: String,

    /// Working directory (absolute path). Defaults to `/` if absent.
    pub cwd: Option<String>,

    /// Environment variables for this command (merged with process env).
    #[serde(default)]
    pub env: HashMap<String, String>,
}
