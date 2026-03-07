// fsn-web – WebUI backend.
// Embedded HTMX + Alpine.js SPA, served from memory (no Node.js build needed).

pub mod api;
pub mod routes;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::Result;
use axum::Router;
use tower_http::trace::TraceLayer;

use crate::api::AppState;

/// Start the management WebUI on `bind:port`.
pub async fn serve(bind: &str, port: u16, fsn_root: &Path) -> Result<()> {
    let state = AppState {
        fsn_root: Arc::new(fsn_root.to_path_buf()),
    };

    let app = Router::new()
        .merge(routes::ui_routes())
        .merge(api::api_routes(state))
        .layer(TraceLayer::new_for_http());

    let addr = format!("{}:{}", bind, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("FSN WebUI listening on http://{}", addr);
    axum::serve(listener, app).await?;
    Ok(())
}

// ── Shared file-finder helpers (used by api.rs) ───────────────────────────────

pub(crate) fn find_project_file(root: &Path) -> Option<PathBuf> {
    let projects = root.join("projects");
    std::fs::read_dir(&projects).ok()?.flatten()
        .filter(|e| e.path().is_dir())
        .flat_map(|d| std::fs::read_dir(d.path()).into_iter().flatten().flatten())
        .map(|e| e.path())
        .find(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".project.toml")
        })
}

pub(crate) fn find_host_file(root: &Path) -> Option<PathBuf> {
    let hosts = root.join("hosts");
    std::fs::read_dir(&hosts).ok()?.flatten()
        .map(|e| e.path())
        .find(|p| {
            let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("");
            name.ends_with(".host.toml") && name != "example.host.toml"
        })
}
