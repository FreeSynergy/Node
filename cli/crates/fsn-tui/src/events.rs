// Keyboard event handling.

use std::path::Path;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::app::{AppState, LogsState, Screen, ServiceStatus};

pub fn handle(key: KeyEvent, state: &mut AppState, root: &Path) -> Result<()> {
    // Global: Ctrl-C / Ctrl-Q always quit
    if key.modifiers.contains(KeyModifiers::CONTROL)
        && (key.code == KeyCode::Char('c') || key.code == KeyCode::Char('q'))
    {
        state.should_quit = true;
        return Ok(());
    }

    // Logs overlay is modal — handle separately
    if state.logs_overlay.is_some() {
        return handle_logs(key, state);
    }

    match state.screen {
        Screen::Welcome  => handle_welcome(key, state, root),
        Screen::Dashboard => handle_dashboard(key, state, root),
    }
}

// ── Welcome screen ────────────────────────────────────────────────────────────

fn handle_welcome(key: KeyEvent, state: &mut AppState, _root: &Path) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        // Tab cycles through language options
        KeyCode::Tab => {
            state.lang = state.lang.toggle();
        }
        // Arrow keys move between buttons
        KeyCode::Left | KeyCode::Right => {
            state.welcome_focus = 1 - state.welcome_focus;
        }
        KeyCode::Enter => {
            if state.welcome_focus == 0 {
                // TODO: launch fsn init inline (Phase 2)
                // For now: signal quit so caller can run `fsn init`
                state.should_quit = true;
            }
            // Button 1 (Open Project) is grayed out — no action
        }
        _ => {}
    }
    Ok(())
}

// ── Dashboard ─────────────────────────────────────────────────────────────────

fn handle_dashboard(key: KeyEvent, state: &mut AppState, _root: &Path) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.should_quit = true;
        }
        KeyCode::Tab => {
            state.lang = state.lang.toggle();
        }
        KeyCode::Up => {
            if state.selected > 0 {
                state.selected -= 1;
            }
        }
        KeyCode::Down => {
            if state.selected + 1 < state.services.len() {
                state.selected += 1;
            }
        }
        KeyCode::Char('l') => {
            // Open logs overlay for selected service
            if let Some(svc) = state.services.get(state.selected) {
                let lines = fetch_logs(&svc.name);
                state.logs_overlay = Some(LogsState {
                    service_name: svc.name.clone(),
                    lines,
                    scroll: 0,
                });
            }
        }
        KeyCode::Char('d') => {
            // Deploy: mark as unknown while running (async in future)
            if let Some(svc) = state.services.get_mut(state.selected) {
                svc.status = ServiceStatus::Unknown;
            }
            // TODO: spawn deploy task
        }
        KeyCode::Char('r') => {
            // Restart selected service via podman
            if let Some(svc) = state.services.get(state.selected) {
                let _ = std::process::Command::new("podman")
                    .args(["restart", &svc.name])
                    .output();
                // Refresh status
                if let Some(row) = state.services.get_mut(state.selected) {
                    row.status = podman_status(&row.name);
                }
            }
        }
        KeyCode::Char('x') => {
            // Remove selected service (stop + remove)
            if let Some(svc) = state.services.get(state.selected) {
                let _ = std::process::Command::new("podman")
                    .args(["stop", &svc.name])
                    .output();
                let _ = std::process::Command::new("podman")
                    .args(["rm", &svc.name])
                    .output();
                state.services.remove(state.selected);
                if state.selected > 0 && state.selected >= state.services.len() {
                    state.selected -= 1;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ── Logs overlay ──────────────────────────────────────────────────────────────

fn handle_logs(key: KeyEvent, state: &mut AppState) -> Result<()> {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            state.logs_overlay = None;
        }
        KeyCode::Up => {
            if let Some(ref mut logs) = state.logs_overlay {
                if logs.scroll > 0 {
                    logs.scroll -= 1;
                }
            }
        }
        KeyCode::Down => {
            if let Some(ref mut logs) = state.logs_overlay {
                let max = logs.lines.len().saturating_sub(1);
                if logs.scroll < max {
                    logs.scroll += 1;
                }
            }
        }
        _ => {}
    }
    Ok(())
}

// ── Podman helpers ────────────────────────────────────────────────────────────

pub fn podman_status(name: &str) -> ServiceStatus {
    let out = std::process::Command::new("podman")
        .args(["inspect", "--format", "{{.State.Status}}", name])
        .output();
    match out {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            let s = s.trim();
            match s {
                "running"  => ServiceStatus::Running,
                "exited" | "stopped" => ServiceStatus::Stopped,
                "error"    => ServiceStatus::Error,
                _          => ServiceStatus::Unknown,
            }
        }
        Err(_) => ServiceStatus::Unknown,
    }
}

fn fetch_logs(name: &str) -> Vec<String> {
    let out = std::process::Command::new("podman")
        .args(["logs", "--tail", "100", name])
        .output();
    match out {
        Ok(o) => {
            let text = if o.stdout.is_empty() { o.stderr } else { o.stdout };
            String::from_utf8_lossy(&text)
                .lines()
                .map(|l| l.to_string())
                .collect()
        }
        Err(_) => vec!["[Logs nicht verfügbar]".into()],
    }
}
