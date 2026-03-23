// Integration test: parse all modules from the Store.
//
// Loads every .toml in FreeSynergy.Store/node/resources/ via ServiceRegistry
// and asserts that key modules are present and well-formed.
//
// The test is skipped gracefully when the store directory does not exist
// (e.g. in CI without a checked-out Store repo).

use std::path::PathBuf;

use fs_node_core::config::registry::ServiceRegistry;

fn store_resources_dir() -> PathBuf {
    // From cli/crates/fs-node-core/ go up 4 levels → /home/kal/Server/
    // then into fs-store/node/resources/
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../../fs-store/node/resources")
}

#[test]
fn all_store_modules_parse_without_error() {
    let dir = store_resources_dir();
    if !dir.exists() {
        eprintln!("SKIP: Store not found at {}", dir.display());
        return;
    }

    let registry = ServiceRegistry::load(&dir).expect("ServiceRegistry::load");
    let classes: Vec<_> = registry.all().collect();

    assert!(!classes.is_empty(), "expected at least one module to be loaded");
    eprintln!("Loaded {} module classes", classes.len());
}

#[test]
fn expected_modules_are_present() {
    let dir = store_resources_dir();
    if !dir.exists() {
        eprintln!("SKIP: Store not found at {}", dir.display());
        return;
    }

    let registry = ServiceRegistry::load(&dir).expect("ServiceRegistry::load");

    let required = [
        "containers/forgejo",
        "apps/kanidm",
        "apps/stalwart",
        "containers/outline",
        "apps/tuwunel",
        "containers/postgres",
        "containers/dragonfly",
        "containers/openobserver",
        "apps/zentinel",
    ];

    for key in &required {
        assert!(
            registry.get(key).is_some(),
            "expected module '{key}' not found in registry"
        );
    }
}

#[test]
fn all_container_modules_have_image() {
    let dir = store_resources_dir();
    if !dir.exists() {
        eprintln!("SKIP: Store not found at {}", dir.display());
        return;
    }

    let registry = ServiceRegistry::load(&dir).expect("ServiceRegistry::load");

    for (key, class) in registry.all() {
        // Native apps have no container block — skip them.
        let Some(container) = &class.container else { continue };

        assert!(
            !container.image.is_empty(),
            "module '{key}' has empty container.image"
        );
        assert!(
            !container.image_tag.is_empty(),
            "module '{key}' has empty container.image_tag"
        );
    }
}

#[test]
fn all_container_modules_have_healthcheck() {
    let dir = store_resources_dir();
    if !dir.exists() {
        eprintln!("SKIP: Store not found at {}", dir.display());
        return;
    }

    let registry = ServiceRegistry::load(&dir).expect("ServiceRegistry::load");

    for (key, class) in registry.all() {
        // Native apps have no container block — skip them.
        let Some(container) = &class.container else { continue };

        assert!(
            container.healthcheck.is_some(),
            "container module '{key}' is missing container.healthcheck (required by convention)"
        );
    }
}

#[test]
fn plugin_dns_and_acme_plugins_parse() {
    let dir = store_resources_dir();
    if !dir.exists() {
        eprintln!("SKIP: Store not found at {}", dir.display());
        return;
    }

    let registry = ServiceRegistry::load(&dir).expect("ServiceRegistry::load");
    let plugins: Vec<_> = registry.all_plugins().collect();

    // At least hetzner + cloudflare + none DNS, and letsencrypt + none ACME
    assert!(plugins.len() >= 5, "expected at least 5 plugins, got {}", plugins.len());

    // New key format: plugins/{plugin_type}/{name}
    // get_plugin(plugin_type, name) — 2 args
    let required_plugins = [
        ("dns", "hetzner"),
        ("dns", "cloudflare"),
        ("acme", "letsencrypt"),
    ];
    for (plugin_type, name) in &required_plugins {
        assert!(
            registry.get_plugin(plugin_type, name).is_some(),
            "expected plugin 'plugins/{plugin_type}/{name}' not found"
        );
    }
}
