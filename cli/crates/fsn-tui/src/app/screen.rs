// Screen and dashboard focus enums.
//
// Pattern: State Machine discriminant — Screen is the top-level state that
// drives which renderer and event handler are active. DashFocus is a
// sub-state within Screen::Dashboard.
//
// NavTab is the new routing enum — it maps directly to what the user sees in
// the nav bar. Screen remains as a legacy alias during migration.

// ── NavTab — primary routing enum ────────────────────────────────────────────

/// Navigation tab — determines which composition pair (sidebar + main) is active.
///
/// This is the Single Source of Truth for app routing. The nav bar renders
/// exactly these variants in order. Tabs marked `is_coming_soon()` are shown
/// dimmed and cannot receive keyboard focus.
///
/// To add a new tab:
///   1. Add a variant here.
///   2. Add a `label_key()` arm.
///   3. Add `is_coming_soon()` arm (false if ready).
///   4. Add sidebar + main composition files.
///   5. Add event handler in events.rs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavTab {
    #[default]
    Projects,    // SidebarProjects + ProjectsMain
    Hosts,       // SidebarHosts + HostsMain
    Services,    // SidebarServices + ServicesMain
    Bots,        // SidebarBots + BotsMain          [active, content coming]
    Federation,  // SidebarFederation + FederationMain  [coming soon]
    Websites,    // SidebarWebsites + WebsitesMain      [coming soon]
    Store,       // SidebarStore + StoreMain
    Settings,    // SidebarSettings + SettingsMain
}

impl NavTab {
    pub const ALL: &'static [NavTab] = &[
        Self::Projects,
        Self::Hosts,
        Self::Services,
        Self::Bots,
        Self::Federation,
        Self::Websites,
        Self::Store,
        Self::Settings,
    ];

    /// i18n key for the nav bar label.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::Projects   => "dash.tab.projects",
            Self::Hosts      => "dash.tab.hosts",
            Self::Services   => "dash.tab.services",
            Self::Bots       => "dash.tab.bots",
            Self::Federation => "dash.tab.federation",
            Self::Websites   => "dash.tab.websites",
            Self::Store      => "dash.tab.store",
            Self::Settings   => "dash.tab.settings",
        }
    }

    /// Tabs that are planned but not yet implemented.
    /// Rendered dimmed in the nav bar; keyboard navigation skips them.
    pub fn is_coming_soon(self) -> bool {
        matches!(self, Self::Federation | Self::Websites | Self::Bots)
    }

    /// Nav bar index (position in the ALL slice).
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap_or(0)
    }

    /// Resolve a NavTab from a nav bar index (0-based).
    pub fn from_index(idx: usize) -> Option<Self> {
        Self::ALL.get(idx).copied()
    }

    /// Next non-coming-soon tab (wraps around).
    pub fn next(self) -> Self {
        let all: Vec<NavTab> = Self::ALL.iter().copied().filter(|t| !t.is_coming_soon()).collect();
        let cur = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(cur + 1) % all.len()]
    }

    /// Previous non-coming-soon tab (wraps around).
    pub fn prev(self) -> Self {
        let all: Vec<NavTab> = Self::ALL.iter().copied().filter(|t| !t.is_coming_soon()).collect();
        let cur = all.iter().position(|&t| t == self).unwrap_or(0);
        all[(cur + all.len() - 1) % all.len()]
    }
}

// ── Screen — legacy state discriminant (kept during migration) ────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Welcome,
    Dashboard,
    /// Form screen — shows the active form from `form_queue`.
    /// Queue tab bar is visible when `form_queue.has_multiple()`.
    NewProject,
    /// Application settings — store management, preferences.
    Settings,
    /// Store browser — browse and install modules from configured stores.
    Store,
}

// ── Dashboard focus ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashFocus {
    Sidebar,
    Services,
}

// ── Settings ──────────────────────────────────────────────────────────────────

/// Which side of the Settings screen has keyboard focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsFocus {
    /// Left sidebar — navigating the section list.
    #[default]
    Sidebar,
    /// Right content panel — navigating items within a section.
    Content,
}

/// Active section within the Settings screen.
///
/// Displayed as a sidebar on the left. Each section renders its own
/// content panel on the right.
///
/// Adding a new section:
///   1. Add a variant here.
///   2. Add a `label_key()` arm.
///   3. Add a render function in `ui/settings_screen.rs`.
///   4. Add a key handler in `events.rs`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SettingsSection {
    #[default]
    General,
    Store,
    Languages,
    About,
}

impl SettingsSection {
    pub const ALL: &'static [SettingsSection] = &[
        Self::General,
        Self::Store,
        Self::Languages,
        Self::About,
    ];

    pub fn from_idx(idx: usize) -> Self {
        Self::ALL.get(idx).copied().unwrap_or_default()
    }

    pub fn idx(self) -> usize {
        Self::ALL.iter().position(|&s| s == self).unwrap_or(0)
    }

    /// i18n key for the sidebar label.
    pub fn label_key(self) -> &'static str {
        match self {
            Self::General   => "settings.section.general",
            Self::Store     => "settings.section.store",
            Self::Languages => "settings.section.languages",
            Self::About     => "settings.section.about",
        }
    }
}

// ── Legacy alias (keeps old code compiling during migration) ──────────────────

/// Kept for backward compatibility — maps to SettingsSection.
pub type SettingsTab = SettingsSection;

// ── Store screen focus ────────────────────────────────────────────────────────

/// Which part of the Settings → Store section has focus.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StoreSettingsFocus {
    #[default]
    Repos,
    Modules,
}

/// Which panel of the Store screen has focus.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StoreScreenFocus {
    #[default]
    Sidebar,
    Detail,
}

/// What the Store screen sidebar is showing.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum StoreSidebarMode {
    #[default]
    ByType,   // grouped by ServiceType category
    All,      // flat list
}
