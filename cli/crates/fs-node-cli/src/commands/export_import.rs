// `fsn export` / `fsn import` — serialize/restore all project configs.
//
// Export bundles project.toml, host.toml, and vault.toml into a single
// TOML document with top-level [project], [host], and [vault] sections.
// Import reads that bundle back out and writes the individual files.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

use fs_node_core::config::find_project;

// ── Bundle type ───────────────────────────────────────────────────────────────

/// A portable TOML bundle of all project configuration files.
///
/// Written by `fsn export` and consumed by `fsn import`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct ConfigBundle {
    /// Raw TOML content of the project config file.
    #[serde(default)]
    pub project: Option<String>,
    /// Raw TOML content of the host config file (if present).
    #[serde(default)]
    pub host: Option<String>,
    /// Raw TOML content of the vault file (if present, **without** secrets if not exported).
    #[serde(default)]
    pub vault: Option<String>,
    /// Relative file paths used to restore individual files.
    #[serde(default)]
    pub project_file: Option<String>,
    #[serde(default)]
    pub host_file: Option<String>,
    #[serde(default)]
    pub vault_file: Option<String>,
}

// ── export ────────────────────────────────────────────────────────────────────

/// Export all project configuration to a single TOML bundle file.
///
/// Reads the project config, associated host config, and vault (if present)
/// and writes them as raw TOML strings into `output`.
pub async fn export(root: &Path, project: Option<&Path>, output: &Path) -> Result<()> {
    let proj_path = project
        .map(std::path::Path::to_path_buf)
        .or_else(|| find_project(root, None))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No project found in '{}'. Use --project to specify one.",
                root.display()
            )
        })?;

    let mut bundle = ConfigBundle::default();

    // Project config.
    let proj_content = std::fs::read_to_string(&proj_path)
        .with_context(|| format!("reading project config: {}", proj_path.display()))?;
    bundle.project = Some(proj_content);
    bundle.project_file = Some(
        proj_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project.toml")
            .to_string(),
    );

    // Determine slug from project file name.
    let slug = proj_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("project")
        .trim_end_matches(".project")
        .to_string();

    // Host config (optional).
    let host_path = root.join("hosts").join(format!("{slug}.host.toml"));
    if host_path.exists() {
        let host_content = std::fs::read_to_string(&host_path)
            .with_context(|| format!("reading host config: {}", host_path.display()))?;
        bundle.host = Some(host_content);
        bundle.host_file = Some(
            host_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("host.toml")
                .to_string(),
        );
    }

    // Vault (optional — warn if secrets are included).
    let proj_dir = proj_path.parent().unwrap_or(root);
    let vault_path = proj_dir.join("vault.toml");
    if vault_path.exists() {
        let vault_content = std::fs::read_to_string(&vault_path)
            .with_context(|| format!("reading vault: {}", vault_path.display()))?;
        eprintln!(
            "WARNING: vault.toml is included in the export bundle. \
             Protect the output file accordingly."
        );
        bundle.vault = Some(vault_content);
        bundle.vault_file = Some("vault.toml".to_string());
    }

    // Serialize bundle.
    let content = toml::to_string_pretty(&bundle).context("serializing config bundle")?;
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("creating output directory: {}", parent.display()))?;
    }
    std::fs::write(output, &content)
        .with_context(|| format!("writing bundle: {}", output.display()))?;

    println!("Exported config bundle to {}", output.display());
    Ok(())
}

// ── import ────────────────────────────────────────────────────────────────────

/// Import configuration from a TOML bundle and write individual config files.
///
/// The project config is written into `{root}/projects/{slug}/`,
/// the host config into `{root}/hosts/`.
pub async fn import(root: &Path, input: &Path) -> Result<()> {
    let content = std::fs::read_to_string(input)
        .with_context(|| format!("reading bundle: {}", input.display()))?;

    let bundle: ConfigBundle =
        toml::from_str(&content).with_context(|| format!("parsing bundle: {}", input.display()))?;

    let mut written = Vec::new();

    // Write project config.
    if let (Some(proj_content), Some(proj_file)) = (&bundle.project, &bundle.project_file) {
        let slug = proj_file
            .trim_end_matches(".project.toml")
            .trim_end_matches(".toml");
        let proj_dir = root.join("projects").join(slug);
        std::fs::create_dir_all(&proj_dir)
            .with_context(|| format!("creating project dir: {}", proj_dir.display()))?;
        let dest = proj_dir.join(proj_file);
        std::fs::write(&dest, proj_content)
            .with_context(|| format!("writing {}", dest.display()))?;
        written.push(dest.display().to_string());
    }

    // Write host config.
    if let (Some(host_content), Some(host_file)) = (&bundle.host, &bundle.host_file) {
        let hosts_dir = root.join("hosts");
        std::fs::create_dir_all(&hosts_dir)
            .with_context(|| format!("creating hosts dir: {}", hosts_dir.display()))?;
        let dest = hosts_dir.join(host_file);
        std::fs::write(&dest, host_content)
            .with_context(|| format!("writing {}", dest.display()))?;
        written.push(dest.display().to_string());
    }

    // Write vault.
    if let (Some(vault_content), Some(vault_file)) = (&bundle.vault, &bundle.vault_file) {
        // Vault lives in the project directory next to the project config.
        if let Some(proj_file) = &bundle.project_file {
            let slug = proj_file
                .trim_end_matches(".project.toml")
                .trim_end_matches(".toml");
            let proj_dir = root.join("projects").join(slug);
            std::fs::create_dir_all(&proj_dir).ok();
            let dest = proj_dir.join(vault_file);
            std::fs::write(&dest, vault_content)
                .with_context(|| format!("writing {}", dest.display()))?;
            written.push(dest.display().to_string());
        }
    }

    if written.is_empty() {
        println!("Bundle contained no config files to import.");
    } else {
        for path in &written {
            println!("  Wrote: {path}");
        }
        println!("Import complete ({} file(s)).", written.len());
    }

    Ok(())
}
