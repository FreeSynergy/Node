// `fsn store` — browse and manage the FreeSynergy module store.
//
// Uses StoreClient (fsn-store) to fetch the Node.Store catalog and display
// available modules. Install/update are delegated to `fsn deploy` once
// the module is added to the project config.

use anyhow::Result;
use fsn_core::store::StoreEntry;
use fsn_store::StoreClient;

// ── search ─────────────────────────────────────────────────────────────────────

/// Search the store catalog for modules matching `query`.
///
/// With an empty query, all modules are listed.
pub async fn search(query: &str) -> Result<()> {
    let mut client = StoreClient::node_store();
    let catalog: fsn_store::Catalog<StoreEntry> = client.fetch_catalog("Node", false).await?;

    let q = query.to_lowercase();
    let matches: Vec<&StoreEntry> = catalog.packages.iter()
        .filter(|e| {
            q.is_empty()
                || e.name.to_lowercase().contains(&q)
                || e.id.to_lowercase().contains(&q)
                || e.description.to_lowercase().contains(&q)
                || e.tags.iter().any(|t| t.to_lowercase().contains(&q))
        })
        .collect();

    if matches.is_empty() {
        if q.is_empty() {
            println!("Store catalog is empty.");
        } else {
            println!("No modules found matching: {query}");
        }
        return Ok(());
    }

    println!("{:<24} {:<10} {}", "ID", "VERSION", "DESCRIPTION");
    println!("{}", "─".repeat(72));
    for entry in &matches {
        let desc = if entry.description.len() > 40 {
            format!("{}…", &entry.description[..39])
        } else {
            entry.description.clone()
        };
        println!("{:<24} {:<10} {}", entry.id, entry.version, desc);
    }
    println!("\n{} module(s) found.", matches.len());
    Ok(())
}

// ── info ───────────────────────────────────────────────────────────────────────

/// Show details for a specific module by ID.
pub async fn info(id: &str) -> Result<()> {
    let mut client = StoreClient::node_store();
    let catalog: fsn_store::Catalog<StoreEntry> = client.fetch_catalog("Node", false).await?;

    let entry = catalog.packages.iter().find(|e| e.id == id);
    match entry {
        None => {
            println!("Module not found: {id}");
            println!("Run `fsn store search` to list available modules.");
        }
        Some(e) => {
            println!("Name:        {}", e.name);
            println!("ID:          {}", e.id);
            println!("Version:     {}", e.version);
            println!("Category:    {}", e.category);
            println!("Description: {}", e.description);
            if let Some(w) = &e.website  { println!("Website:     {w}"); }
            if let Some(r) = &e.repository { println!("Repository:  {r}"); }
            if let Some(l) = &e.license  { println!("License:     {l}"); }
            if !e.tags.is_empty()         { println!("Tags:        {}", e.tags.join(", ")); }
        }
    }
    Ok(())
}

// ── install ────────────────────────────────────────────────────────────────────

/// Install a module by adding it to the project config.
///
/// This prints instructions; actual deployment is done via `fsn deploy`.
pub async fn install(id: &str) -> Result<()> {
    let mut client = StoreClient::node_store();
    let catalog: fsn_store::Catalog<StoreEntry> = client.fetch_catalog("Node", false).await?;

    if catalog.packages.iter().find(|e| e.id == id).is_none() {
        println!("Module not found: {id}");
        println!("Run `fsn store search` to list available modules.");
        return Ok(());
    }

    println!("To install '{id}', add it to your project config:");
    println!();
    println!("  [load.services.my-{id}]");
    println!("  service_class = \"{id}\"");
    println!();
    println!("Then run `fsn deploy` to apply.");
    Ok(())
}

// ── update ─────────────────────────────────────────────────────────────────────

/// Check for module updates and report available newer versions.
pub async fn update_check() -> Result<()> {
    let mut client = StoreClient::node_store();
    let catalog: fsn_store::Catalog<StoreEntry> = client.fetch_catalog("Node", false).await?;

    println!("Fetched catalog: {} modules available.", catalog.packages.len());
    println!("To update a deployed module, run `fsn update --service <name>`.");
    Ok(())
}
