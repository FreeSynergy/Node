// fsn-tui — Terminal UI for FreeSynergy.Node.
//
// Entry point: `run(root)` — called by `fsn tui`.
// Detects whether a project exists → Welcome screen or Dashboard.

pub mod app;
pub mod events;
pub mod i18n;
pub mod sysinfo;
pub mod ui;

use std::io;
use std::path::Path;

use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};

use app::{AppState, ProjectInfo, ServiceRow, ServiceStatus};
use sysinfo::SysInfo;

/// Start the TUI. Blocks until the user quits.
pub fn run(root: &Path) -> Result<()> {
    // Collect system info (may take a moment for Podman version)
    let sysinfo = SysInfo::collect();

    // Load running services from Podman (best-effort)
    let services = load_services(root);

    let projects = load_projects(root);
    let mut state = AppState::new(sysinfo, services, projects);

    // If a project.toml already exists, go straight to Dashboard
    // even when no containers are running yet
    if state.screen == app::Screen::Welcome && project_toml_exists(root) {
        state.screen = app::Screen::Dashboard;
    }

    // Terminal setup
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run event loop
    let result = app::run_loop(&mut terminal, &mut state, root);

    // Restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    result
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Load all projects from `root/projects/` into ProjectInfo structs.
pub fn load_projects(root: &Path) -> Vec<ProjectInfo> {
    let projects_dir = root.join("projects");
    if !projects_dir.exists() { return vec![]; }

    let mut projects = Vec::new();
    let Ok(entries) = std::fs::read_dir(&projects_dir) else { return projects; };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let Ok(inner) = std::fs::read_dir(&path) else { continue; };
        for f in inner.flatten() {
            let fp = f.path();
            if fp.extension().and_then(|e| e.to_str()) != Some("toml") { continue; }
            if !fp.file_stem().and_then(|s| s.to_str())
                .map(|s| s.ends_with(".project"))
                .unwrap_or(false) { continue; }
            if let Some(info) = parse_project_toml(&fp) {
                projects.push(info);
            }
        }
    }
    projects
}

fn parse_project_toml(path: &Path) -> Option<ProjectInfo> {
    let content = std::fs::read_to_string(path).ok()?;
    let val: toml::Value = content.parse().ok()?;
    let proj = val.get("project")?;

    let get = |key: &str| proj.get(key).and_then(|v| v.as_str()).unwrap_or("").to_string();

    let stem = path.file_stem()?.to_str()?;
    let slug = stem.strip_suffix(".project").unwrap_or(stem).to_string();

    Some(ProjectInfo {
        slug,
        name:        get("name"),
        domain:      get("domain"),
        description: get("description"),
        email:       get("email"),
        language:    get("language"),
        version:     get("version"),
        path:        get("path"),
        toml_path:   path.to_path_buf(),
    })
}

/// Returns true if any *.project.toml exists under `root/projects/`.
fn project_toml_exists(root: &Path) -> bool {
    let projects_dir = root.join("projects");
    if !projects_dir.exists() { return false; }
    let Ok(entries) = std::fs::read_dir(&projects_dir) else { return false; };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }
        let Ok(inner) = std::fs::read_dir(&path) else { continue; };
        for f in inner.flatten() {
            let fp = f.path();
            if fp.extension().and_then(|e| e.to_str()) == Some("toml")
                && fp.file_stem().and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".project"))
                    .unwrap_or(false)
            {
                return true;
            }
        }
    }
    false
}

/// Try to load active services from the project config.
/// Returns empty Vec if no project found — triggers Welcome screen.
fn load_services(root: &Path) -> Vec<ServiceRow> {
    let mut rows = Vec::new();

    // Look for *.project.toml files in projects/
    let projects_dir = root.join("projects");
    if !projects_dir.exists() {
        return rows;
    }

    // Quick scan: find any project directory with a .project.toml
    let Ok(entries) = std::fs::read_dir(&projects_dir) else { return rows };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() { continue; }

        // Find *.project.toml in this dir
        let Ok(inner) = std::fs::read_dir(&path) else { continue };
        for f in inner.flatten() {
            let fp = f.path();
            if fp.extension().and_then(|e| e.to_str()) == Some("toml")
                && fp.file_stem().and_then(|s| s.to_str())
                    .map(|s| s.ends_with(".project"))
                    .unwrap_or(false)
            {
                // Found a project — try to read service list from Podman
                rows.extend(podman_service_rows());
                return rows;
            }
        }
    }

    rows
}

/// Query Podman for all FSN-managed containers and their status.
fn podman_service_rows() -> Vec<ServiceRow> {
    let out = std::process::Command::new("podman")
        .args(["ps", "-a", "--format", "{{.Names}}|{{.Status}}"])
        .output();

    let Ok(output) = out else { return vec![] };
    let text = String::from_utf8_lossy(&output.stdout);

    text.lines()
        .filter_map(|line| {
            let mut parts = line.splitn(2, '|');
            let name   = parts.next()?.trim().to_string();
            let status = parts.next().unwrap_or("").trim();
            if name.is_empty() { return None; }

            let svc_status = if status.starts_with("Up") {
                ServiceStatus::Running
            } else if status.starts_with("Exited") {
                ServiceStatus::Stopped
            } else {
                ServiceStatus::Unknown
            };

            Some(ServiceRow {
                domain:       format!("{}.example.com", name),
                service_type: "custom".into(),
                name,
                status:       svc_status,
            })
        })
        .collect()
}
