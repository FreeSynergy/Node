// Screen and dashboard focus enums.
//
// Pattern: State Machine discriminant — Screen is the top-level state that
// drives which renderer and event handler are active. DashFocus is a
// sub-state within Screen::Dashboard.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Dashboard,
    /// Form screen — shows the active form from `form_queue`.
    /// Queue tab bar is visible when `form_queue.has_multiple()`.
    NewProject,
    /// Application settings — store management, preferences.
    Settings,
}

// ── Dashboard focus ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashFocus {
    Sidebar,
    Services,
}

// ── Settings tabs ─────────────────────────────────────────────────────────────

/// Active tab within the Settings screen.
/// Add new variants here as settings grow — the screen renders the correct
/// section and the event handler routes keys accordingly.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsTab {
    #[default]
    Stores,
    Languages,
}

impl SettingsTab {
    /// Cycle to the next tab (wraps around).
    pub fn next(self) -> Self {
        match self { Self::Stores => Self::Languages, Self::Languages => Self::Stores }
    }

    /// i18n key for the tab label shown in the tab bar.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Stores    => "settings.tab.stores",
            Self::Languages => "settings.tab.languages",
        }
    }
}
