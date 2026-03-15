// `fsn install <package> [--dry-run]` — add a package from the store to the project.
//
// Flow:
//   1. Fetch the Node store catalog.
//   2. Find the package by ID.
//   3. Check if it is already declared in the project config.
//   4. Dry-run: print what would happen.
//   5. Otherwise: append a [load.services.{name}] block and confirm.

use anyhow::{Context, Result};
use std::path::Path;

use fsn_node_core::config::{find_project, ProjectConfig};
use fsn_node_core::store::StoreEntry;
use fsn_store::StoreClient;

// ── run ───────────────────────────────────────────────────────────────────────

/// Install a package from the store into the current project.
///
/// With `dry_run = true` nothing is written; the planned changes are printed
/// instead so the user can review them before committing.
pub async fn run(root: &Path, package: &str, dry_run: bool) -> Result<()> {
    // 1. Fetch store catalog.
    let mut client = StoreClient::node_store();
    let catalog: fsn_store::Catalog<StoreEntry> = client
        .fetch_catalog("Node", false)
        .await
        .map_err(anyhow::Error::from)
        .context("fetching module catalog")?;

    // 2. Find package by ID.
    let entry: &StoreEntry = catalog
        .packages
        .iter()
        .find(|e| e.id == package)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Package '{}' not found in the store catalog. \
                 Run `fsn store search` to list available packages.",
                package
            )
        })?;

    // 3. Locate project config.
    let proj_path = find_project(root, None)
        .ok_or_else(|| anyhow::anyhow!("No project found in '{}'. Run `fsn init` first.", root.display()))?;

    let project = ProjectConfig::load(&proj_path)
        .with_context(|| format!("loading project config: {}", proj_path.display()))?;

    // 4. Check if package is already declared.
    let already_installed = project
        .load
        .services
        .values()
        .any(|svc| svc.service_class == package);

    if already_installed {
        println!(
            "Package '{}' is already declared in your project config.",
            package
        );
        println!("Run `fsn deploy` to ensure it is deployed.");
        return Ok(());
    }

    // Derive a sensible instance name from the package ID (e.g. "git/forgejo" → "forgejo").
    let instance_name = package
        .split('/')
        .last()
        .unwrap_or(package)
        .to_string();

    let block = format!(
        "\n[load.services.{instance}]\nservice_class = \"{pkg}\"\nversion       = \"{ver}\"\n",
        instance = instance_name,
        pkg      = entry.id,
        ver      = entry.version,
    );

    // 5a. Dry-run: print the plan.
    if dry_run {
        println!("Dry-run: the following block would be appended to {}", proj_path.display());
        println!("{}", block);
        println!("No changes were written.");
        return Ok(());
    }

    // 5b. Append to the project config file.
    let current = std::fs::read_to_string(&proj_path)
        .with_context(|| format!("reading {}", proj_path.display()))?;

    let updated = format!("{}{}", current.trim_end_matches('\n'), block);

    std::fs::write(&proj_path, updated)
        .with_context(|| format!("writing {}", proj_path.display()))?;

    println!(
        "Added '{}' as instance '{}' to {}",
        entry.id,
        instance_name,
        proj_path.display()
    );
    println!("Run `fsn deploy` to apply.");
    Ok(())
}
