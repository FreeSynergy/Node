// generate.rs — Generate FSN module TOML from a detected ComposeService.

use std::collections::HashMap;
use std::fmt::Write as _;

use crate::compose::ComposeService;
use crate::detect::ServiceTypeHint;

// ── ModuleToml ────────────────────────────────────────────────────────────────

/// Generated FSN module definition.
#[derive(Debug, Clone)]
pub struct ModuleToml {
    pub name: String,
    pub class: String,
    pub image: String,
    pub description: String,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub env: HashMap<String, String>,
    pub health_path: Option<String>,
}

impl ModuleToml {
    /// Serialise to TOML text in FSN module format.
    #[must_use]
    pub fn to_toml(&self) -> String {
        let mut out = String::new();

        out.push_str("[module]\n");
        let _ = writeln!(out, "name        = {:?}", self.name);
        let _ = writeln!(out, "class       = {:?}", self.class);
        let _ = writeln!(out, "description = {:?}", self.description);
        out.push('\n');

        out.push_str("[container]\n");
        let _ = writeln!(out, "image = {:?}", self.image);

        if let Some(hp) = &self.health_path {
            let _ = writeln!(out, "health_path = {hp:?}");
        }

        // healthcheck block
        out.push('\n');
        out.push_str("[container.healthcheck]\n");
        out.push_str("test     = [\"CMD\", \"curl\", \"-f\", \"http://localhost/health\"]\n");
        out.push_str("interval = \"30s\"\n");
        out.push_str("timeout  = \"10s\"\n");
        out.push_str("retries  = 3\n");

        if !self.ports.is_empty() {
            out.push('\n');
            out.push_str("[container.published_ports]\n");
            for p in &self.ports {
                let _ = writeln!(out, "# {p}");
            }
        }

        if !self.volumes.is_empty() {
            out.push('\n');
            out.push_str("# volumes:\n");
            for v in &self.volumes {
                let _ = writeln!(out, "#   {v}");
            }
        }

        if !self.env.is_empty() {
            out.push('\n');
            out.push_str("[environment]\n");
            let mut keys: Vec<_> = self.env.keys().collect();
            keys.sort();
            for k in keys {
                let v = &self.env[k];
                let _ = writeln!(out, "{k} = {v:?}");
            }
        }

        out
    }
}

// ── Generator ─────────────────────────────────────────────────────────────────

/// Generate a `ModuleToml` from a `ComposeService` and its detected type hint.
#[must_use]
pub fn generate(svc: &ComposeService, hint: &ServiceTypeHint) -> ModuleToml {
    let class = if hint.class == "unknown" {
        // Default fallback
        "proxy/zentinel".to_owned()
    } else {
        hint.class.clone()
    };

    let health_path = guess_health_path(&class);

    ModuleToml {
        name: svc.name.clone(),
        class,
        image: svc.image.clone(),
        description: format!("Auto-generated from Docker Compose service '{}'", svc.name),
        ports: svc.ports.clone(),
        volumes: svc.volumes.clone(),
        env: svc.env.clone(),
        health_path,
    }
}

fn guess_health_path(class: &str) -> Option<String> {
    match class.split('/').next().unwrap_or("") {
        "mail" => None,
        "wiki" => Some("/healthcheck".to_owned()),
        "iam" => Some("/status".to_owned()),
        "monitoring" => Some("/-/health".to_owned()),
        _ => Some("/health".to_owned()),
    }
}
