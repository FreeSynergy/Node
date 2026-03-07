// Application state and main event loop.

use std::path::Path;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::sysinfo::SysInfo;
use crate::ui;

// ── Types ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Dashboard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    De,
    En,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::De => Lang::En,
            Lang::En => Lang::De,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::De => "DE",
            Lang::En => "EN",
        }
    }
}

/// A single row in the services table.
#[derive(Debug, Clone)]
pub struct ServiceRow {
    pub name:        String,
    pub service_type: String,
    pub domain:      String,
    pub status:      ServiceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Error,
    Unknown,
}

impl ServiceStatus {
    pub fn i18n_key(self) -> &'static str {
        match self {
            ServiceStatus::Running => "status.running",
            ServiceStatus::Stopped => "status.stopped",
            ServiceStatus::Error   => "status.error",
            ServiceStatus::Unknown => "status.unknown",
        }
    }
}

/// Full application state — passed to every render and event handler.
pub struct AppState {
    pub screen:             Screen,
    pub lang:               Lang,
    pub sysinfo:            SysInfo,
    pub services:           Vec<ServiceRow>,
    pub selected:           usize,
    pub logs_overlay:       Option<LogsState>,
    pub lang_dropdown_open: bool,
    pub should_quit:        bool,
    /// Which button is focused on the welcome screen (0=New, 1=Open)
    pub welcome_focus:      usize,
    last_refresh:           Instant,
}

#[derive(Debug, Clone)]
pub struct LogsState {
    pub service_name: String,
    pub lines:        Vec<String>,
    pub scroll:       usize,
}

// ── Constructor ───────────────────────────────────────────────────────────────

impl AppState {
    pub fn new(sysinfo: SysInfo, services: Vec<ServiceRow>) -> Self {
        let screen = if services.is_empty() {
            Screen::Welcome
        } else {
            Screen::Dashboard
        };
        Self {
            screen,
            lang: Lang::De,
            sysinfo,
            services,
            selected: 0,
            logs_overlay: None,
            lang_dropdown_open: false,
            should_quit: false,
            welcome_focus: 0,
            last_refresh: Instant::now(),
        }
    }

    pub fn t<'a>(&self, key: &'a str) -> &'a str {
        crate::i18n::t(self.lang, key)
    }
}

// ── Main loop ─────────────────────────────────────────────────────────────────

pub fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    state: &mut AppState,
    root: &Path,
) -> Result<()> {
    const POLL_MS: u64 = 250;
    const REFRESH_SECS: u64 = 5;

    loop {
        terminal.draw(|f| ui::render(f, state))?;

        if event::poll(Duration::from_millis(POLL_MS))? {
            if let Event::Key(key) = event::read()? {
                crate::events::handle(key, state, root)?;
            }
        }

        if state.should_quit {
            break;
        }

        // Periodic sysinfo refresh
        if state.last_refresh.elapsed() >= Duration::from_secs(REFRESH_SECS) {
            state.sysinfo = SysInfo::collect();
            state.last_refresh = Instant::now();
        }
    }

    Ok(())
}
