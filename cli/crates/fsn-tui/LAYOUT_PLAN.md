# FSN TUI — Layout & Composition Plan

## Kernprinzip: Immer dasselbe Layout

Das Layout ist **unveränderlich**. Egal ob Dashboard, Form, Store, Settings —
der Rahmen bleibt **immer identisch**. Nur die *Compositions* (Inhalts-Slots)
wechseln.

```
┌──────────────────────────────────────────────────────────────────────────┐
│                                                                          │
│  HEADER   [Logo + Titel + Projekt-Kontext]           [LangSwitch]  [?]  │  ← 6 Zeilen, immer
│                                                                          │
├──────────────────────────────────────────────────────────────────────────┤
│  NAV-BAR  [Projekte] [Hosts] [Services] [Bots] [Federation] [Store] [⚙] │  ← 1 Zeile, immer
├──────────────┬───────────────────────────────────────────────────────────┤
│              │                                                            │
│  LEFT PANEL  │               MAIN AREA                      [RIGHT]      │
│  (optional)  │               (immer vorhanden)              (optional)   │
│              │                                                            │
├──────────────┴───────────────────────────────────────────────────────────┤
│  FOOTER   [Copyright]                          [Kontext-Shortcuts]       │  ← 1 Zeile, immer
└──────────────────────────────────────────────────────────────────────────┘
```

Der `Screen`-enum entfällt als Renderer-Dispatcher. Stattdessen:
- **LayoutSlots** beschreiben, welche Composition in welchem Slot sitzt.
- Der Render-Loop kennt nur ein einziges Top-Level-Layout.

---

## Slots (Zone-Bezeichnungen)

| Slot          | Immer da? | Größe            | Beschreibung                          |
|---------------|-----------|------------------|---------------------------------------|
| `header`      | Ja        | 6 Zeilen fix     | Logo, Titel, Projekt-Kontext          |
| `navbar`      | Ja        | 1 Zeile fix      | Tab-Navigationsleiste                 |
| `left`        | Optional  | 28 Zeichen fix   | Sidebar / Navigationsbaum             |
| `main`        | Ja        | `Min(1)`         | Hauptinhalt (wechselbar)              |
| `right`       | Optional  | variabel         | Hilfe-Panel, Detailansicht extra      |
| `footer`      | Ja        | 1 Zeile fix      | Copyright + Kontext-Shortcuts         |

> **Header-Größe**: 6 Zeilen gibt mehr Luft. Die Compositions dahinter bleiben
> dieselben — später kann man die Zeilenzahl anpassen ohne die Compositions
> zu ändern (`LayoutConfig::topbar_height` ist der einzige Griff dafür).

---

## Compositions (vollständige Liste)

Eine Composition ist ein `struct` das `Composition` implementiert:

```rust
pub trait Composition {
    fn render(&self, f: &mut RenderCtx<'_>, area: Rect, state: &mut AppState);
    fn handle_key(&self, key: KeyEvent, state: &mut AppState) -> bool;
    fn handle_mouse(&self, event: MouseEvent, state: &mut AppState) -> bool;
}
```

### Header-Slot Compositions

| ID                 | Datei                            | Beschreibung                             |
|--------------------|----------------------------------|------------------------------------------|
| `HeaderMain`       | `ui/compositions/header_main.rs` | Logo + FreeSynergy.Node + Build-Info     |
| `HeaderLangSwitch` | `ui/compositions/header_lang.rs` | Sprachumschalter [DE] / [EN] (rechts)    |

> Header ist intern aufgeteilt: links = `HeaderMain`, rechts = `HeaderLangSwitch`.
> Beide sind immer aktiv — kein Wechsel möglich.

### Navbar-Slot Compositions

| ID           | Datei                            | Beschreibung                               |
|--------------|----------------------------------|--------------------------------------------|
| `NavBarMain` | `ui/compositions/navbar_main.rs` | Tab-Leiste: alle NavTabs mit Aktivmarkierung |

> Navbar zeigt immer alle Tabs. Aktiver Tab = `AppState::active_tab`.
> Tabs die "coming soon" sind werden gedimmt dargestellt (nicht deaktiviert).

### Left-Slot Compositions

| ID                  | Datei                                     | Beschreibung                          |
|---------------------|-------------------------------------------|---------------------------------------|
| `SidebarProjects`   | `ui/compositions/sidebar_projects.rs`     | Projektliste mit Health-Indikatoren   |
| `SidebarHosts`      | `ui/compositions/sidebar_hosts.rs`        | Host-Liste mit Health-Indikatoren     |
| `SidebarServices`   | `ui/compositions/sidebar_services.rs`     | Service-Liste mit Status-Icons        |
| `SidebarBots`       | `ui/compositions/sidebar_bots.rs`         | Bot-Liste                             |
| `SidebarFederation` | `ui/compositions/sidebar_federation.rs`   | Federation-Knoten-Liste               |
| `SidebarWebsites`   | `ui/compositions/sidebar_websites.rs`     | Website-/Landingpage-Liste            |
| `SidebarStore`      | `ui/compositions/sidebar_store.rs`        | Store-Kategorien / Modul-Liste        |
| `SidebarSettings`   | `ui/compositions/sidebar_settings.rs`     | Einstellungs-Sektionen                |

> Welche Sidebar aktiv ist, folgt aus `AppState::active_tab`.

### Main-Slot Compositions

| ID                | Datei                                    | Beschreibung                            |
|-------------------|------------------------------------------|-----------------------------------------|
| `ProjectsMain`    | `ui/compositions/projects_main.rs`       | Projekt-Detail / Übersicht              |
| `HostsMain`       | `ui/compositions/hosts_main.rs`          | Host-Detail / Metriken                  |
| `ServicesMain`    | `ui/compositions/services_main.rs`       | Services-Tabelle + Detail               |
| `BotsMain`        | `ui/compositions/bots_main.rs`           | Bot-Konfiguration + Status              |
| `FederationMain`  | `ui/compositions/federation_main.rs`     | Federation-Übersicht (coming soon)      |
| `WebsitesMain`    | `ui/compositions/websites_main.rs`       | Websites / Landingpages (coming soon)   |
| `StoreMain`       | `ui/compositions/store_main.rs`          | Store-Browser mit Modul-Detail          |
| `SettingsMain`    | `ui/compositions/settings_main.rs`       | Einstellungs-Inhalt (General/Langs/...) |
| `FormMain`        | `ui/compositions/form_main.rs`           | Universelle Form-Anzeige (ResourceForm) |
| `TaskWizardMain`  | `ui/compositions/task_wizard_main.rs`    | Deploy-Wizard                           |
| `LogsMain`        | `ui/compositions/logs_main.rs`           | Log-Viewer (Container-Logs)             |

> **Kein `WelcomeMain`-Composition mehr** — der Welcome-Zustand ist ein Overlay
> (Popup) über dem normalen Layout. Siehe Abschnitt "Welcome-Overlay" unten.

### Right-Slot Compositions (optional)

| ID            | Datei                               | Beschreibung                           |
|---------------|-------------------------------------|----------------------------------------|
| `HelpSidebar` | `ui/compositions/help_sidebar.rs`   | F1 Kontext-Hilfe                       |
| `DetailExtra` | `ui/compositions/detail_extra.rs`   | Erweitertes Detail-Panel (z.B. Env)    |

> `right = None` → kein Panel (kein extra Struct nötig).

### Footer-Slot Compositions

| ID           | Datei                            | Beschreibung                              |
|--------------|----------------------------------|-------------------------------------------|
| `FooterMain` | `ui/compositions/footer_main.rs` | Copyright links + Shortcuts rechts        |

> Footer ist immer `FooterMain`. Shortcuts sind kontextabhängig (aus AppState).

---

## Aktive Slot-Belegung pro Tab/Zustand

```
Tab/Zustand        | left                | main              | right (optional)
-------------------|---------------------|-------------------|-----------------
Projekte           | SidebarProjects     | ProjectsMain      | HelpSidebar?
Hosts              | SidebarHosts        | HostsMain         | HelpSidebar?
Services           | SidebarServices     | ServicesMain      | HelpSidebar?
Bots               | SidebarBots         | BotsMain          | —
Federation         | SidebarFederation   | FederationMain    | —
Websites           | SidebarWebsites     | WebsitesMain      | —
Store              | SidebarStore        | StoreMain         | —
Einstellungen      | SidebarSettings     | SettingsMain      | —
Form aktiv         | SidebarXxx* (dim)   | FormMain          | HelpSidebar?
Deploy-Wizard      | SidebarXxx* (dim)   | TaskWizardMain    | —
Logs               | SidebarServices*(d) | LogsMain          | —
```

`* (dim)` = Sidebar bleibt sichtbar, aber fokus-inaktiv (gedimmt dargestellt).
Die Sidebar-Composition weiß selbst ob sie gedimmt ist (`state.form_queue.is_some()`).

---

## Welcome-Overlay (kein eigener Screen mehr)

Wenn `state.projects.is_empty()` → wird **zusätzlich** ein zentriertes Popup
als `OverlayLayer::Welcome` über dem normalen Layout gezeichnet.

```
┌──────────────────────────────────────────────────────────────────────────┐
│  HEADER (normal)                                                         │
├──────────────────────────────────────────────────────────────────────────┤
│  NAV-BAR (normal, Projects aktiv)                                        │
├──────────────┬───────────────────────────────────────────────────────────┤
│              │   ╔══════════════════════════════════════════════════╗    │
│  Sidebar     │   ║  Willkommen bei FreeSynergy.Node                 ║    │
│  (leer,      │   ║  Dezentrale Infrastruktur — frei und selbst...   ║    │
│   gedimmt)   │   ║                                                   ║    │
│              │   ║  ┌─ System ──────────────────────────────────┐   ║    │
│              │   ║  │  Host: ...   Podman: ...                  │   ║    │
│              │   ║  └───────────────────────────────────────────┘   ║    │
│              │   ║                                                   ║    │
│              │   ║  [ Neues Projekt ]    [ Projekt öffnen ]         ║    │
│              │   ╚══════════════════════════════════════════════════╝    │
├──────────────┴───────────────────────────────────────────────────────────┤
│  FOOTER (normal)                                                         │
└──────────────────────────────────────────────────────────────────────────┘
```

**Vorteile:**
- Layout ist wirklich 100% immer gleich
- Welcome-Inhalt (Sysinfo, Buttons) bleibt in `ui/overlays/welcome_overlay.rs`
- Kein duplizierter Header/Footer-Code mehr

---

## NavTab (ersetzt Screen-Enum für Routing)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum NavTab {
    #[default]
    Projects,    // zeigt SidebarProjects + ProjectsMain
    Hosts,       // zeigt SidebarHosts + HostsMain
    Services,    // zeigt SidebarServices + ServicesMain
    Bots,        // zeigt SidebarBots + BotsMain
    Federation,  // zeigt SidebarFederation + FederationMain  [coming later]
    Websites,    // zeigt SidebarWebsites + WebsitesMain      [coming later]
    Store,       // zeigt SidebarStore + StoreMain
    Settings,    // zeigt SidebarSettings + SettingsMain
}

impl NavTab {
    /// True for tabs that are planned but not yet implemented.
    /// NavBar renders these dimmed; keyboard navigation skips them.
    pub fn is_coming_soon(self) -> bool {
        matches!(self, Self::Federation | Self::Websites)
    }
}
```

`NavTab` bestimmt:
1. Welche Sidebar-Composition im `left`-Slot sitzt
2. Welche Main-Composition im `main`-Slot sitzt
3. Welche Shortcuts im Footer erscheinen

---

## Store-Tab — gleich wie alle anderen Tabs

Store verhält sich identisch zu Projekte/Hosts/Services:
- `SidebarStore` links: Kategorien (Proxy, IAM, Mail, Git, Wiki, …) + Suchfeld
- `StoreMain` rechts: Modul-Liste für gewählte Kategorie + Modul-Detail-Panel
- Installieren-Aktion öffnet `FormMain` (Service-Form mit vorausgefüllten Werten)
- Sidebar bleibt gedimmt sichtbar wenn Install-Form offen ist

---

## Maus-Konsistenz-Regeln

1. **Click-Map immer vollständig**: Jede Composition registriert ihre Rects im `ClickMap`.
2. **Einheitlicher Hit-Test**: `mouse.rs` prüft zuerst Overlays, dann Right, dann Main, dann Left, dann NavBar, dann Header/Footer.
3. **Hover-State**: `AppState::hovered: Option<HoverTarget>` — jede Composition setzt beim Rendern Hover-Rects.
4. **Right-Click überall**: Jede Composition liefert `context_actions()` für das ContextMenu.
5. **Scroll**: Scrollable Compositions implementieren `handle_scroll()` — kein globales Scroll-Handling.
6. **NavBar Klick**: Klick auf Tab-Label → `AppState::active_tab` setzen + ClickMap-Region der NavBar.

---

## Was sich ändert gegenüber heute

### Was wegfällt
- `Screen::Welcome` als separater Renderer mit eigenem Header/Footer
- `Screen::NewProject` als separater Screen (→ wird `FormMain` im Layout)
- `Screen::Settings` / `Screen::Store` als separate Screens
- `ui/welcome.rs` eigene Header/Footer-Logik (dupliziert!)
- Alle `if state.screen == Screen::X { render_X() }` Dispatcher

### Was entsteht
- `ui/compositions/` — alle Compositions als einzelne Dateien
- `ui/overlays/welcome_overlay.rs` — Welcome als Overlay
- `Composition` trait in `ui/compositions/mod.rs`
- `LayoutSlots::from_state(state)` → aktive Compositions aus Tab + Zustand
- `ui/root.rs` — einziger Render-Einstieg

### Was bleibt (unverändert)
- `AppLayout` + `LayoutConfig` in `ui/layout.rs`
- `Component` trait + low-level Components (werden interne Helfer der Compositions)
- `OverlayStack` — Overlays bleiben Overlays (ConfirmDialog, ContextMenu, Notifications, Welcome)
- Alle `*_form.rs` — `FormMain` wraps sie, kein Refactor der Forms nötig
- `events.rs` Chain-of-Responsibility

---

## Implementierungs-Reihenfolge (für Sonnet)

1. `NavTab` enum in `app/screen.rs`, `AppState::active_tab` anlegen
2. `Composition` trait in `ui/compositions/mod.rs`
3. `LayoutSlots::from_state()` + `ui/root.rs` (einziger Render-Einstieg)
4. `NavBarMain` aus `header_bar.rs` Tab-Teil extrahieren
5. `OverlayLayer::Welcome` + `ui/overlays/welcome_overlay.rs` (aus `welcome.rs` migrieren)
6. `ProjectsMain` / `HostsMain` / `ServicesMain` aus `ui/detail.rs` extrahieren
7. `StoreMain` + `SidebarStore` aus `ui/store_screen.rs`
8. `SettingsMain` + `SidebarSettings` aus `ui/settings_screen.rs`
9. `BotsMain` + `SidebarBots` (Stub: "coming soon" Text)
10. `FederationMain` + `WebsitesMain` + jeweilige Sidebars (Stub)
11. `FormMain` wraps bestehende `ResourceForm`
12. Maus-Konsistenz-Pass: alle Compositions ClickMap-Rects
13. `Screen` enum entfernen (Legacy-Alias bis alles migriert)

---

## Dateistruktur nach Migration

```
ui/
  compositions/
    mod.rs                  ← Composition trait + LayoutSlots
    header_main.rs
    header_lang.rs
    navbar_main.rs
    sidebar_projects.rs
    sidebar_hosts.rs
    sidebar_services.rs
    sidebar_bots.rs
    sidebar_federation.rs   ← Stub
    sidebar_websites.rs     ← Stub
    sidebar_store.rs
    sidebar_settings.rs
    projects_main.rs
    hosts_main.rs
    services_main.rs
    bots_main.rs
    federation_main.rs      ← Stub
    websites_main.rs        ← Stub
    store_main.rs
    settings_main.rs
    form_main.rs
    task_wizard_main.rs
    logs_main.rs
    help_sidebar.rs
    footer_main.rs
  overlays/
    mod.rs
    welcome_overlay.rs      ← aus welcome.rs migriert
    confirm_dialog.rs       ← aus overlay.rs extrahiert
    context_menu.rs         ← aus overlay.rs extrahiert
    notif_stack.rs          ← aus components/ verschoben
  components/               ← nur noch low-level Helfer
    mod.rs
    notif_stack.rs          ← bleibt bis nach overlays/ Migration
  layout.rs                 ← unverändert
  root.rs                   ← NEU: einziger Render-Einstieg
  dashboard.rs              ← wird zu root.rs migriert → löschen
  welcome.rs                ← wird zu overlays/welcome_overlay.rs → löschen
  ...
```

---

*Zuletzt aktualisiert: 2026-03-13 — Entscheidungen eingetragen*
