# FreeSynergy — Architecture

---

## 1. Grundprinzipien

### 1.1 Code-Wiederverwendung

Jede Funktionalität wird zuerst als eigenständige Library gebaut. `fs-*` Libraries in FreeSynergy.Lib wissen nichts von FreeSynergy.Node — sie sind für Wiki.rs, Decidim.rs und jeden anderen nutzbar.

### 1.2 Standards

- **WASM-First** für Plugins (wasmtime + wit-bindgen Component Model)
- **CRDT von Tag 1** (Automerge)
- **ActivityPub von Tag 1** (activitypub_federation)
- **Nur Tera** für Templates
- **Englisch** im Code und in Kommentaren
- **Deutsch** in der Kommunikation hier und in Claude Code

### 1.3 UX-Konsistenz

Desktop, Web und TUI müssen sich **gleich anfühlen**. Gleiche Navigation, gleiche Shortcuts, gleiche Fenster-Metapher.

### 1.4 Universelle Befehlsschnittstelle

Jeder Befehl ist über **alle drei Interfaces** nutzbar:
- **CLI**: `fsn conductor start <service>`
- **TUI**: `fsn conductor` (Dioxus terminal)
- **GUI**: `fs-conductor` (Dioxus desktop/web)

Core-Logik liegt in der Library. Die drei Frontends sind dünne Wrapper.

---

## 2. Repository-Struktur

```
FreeSynergy/Lib            ← Wiederverwendbare Bibliotheken (Cargo Workspace)      [eigenes Repo]
FreeSynergy/Node           ← CLI + Deployment-Engine (Cargo Workspace)             [eigenes Repo]
FreeSynergy/Desktop        ← Desktop-Umgebung (Cargo Workspace, nutzt fs-*)       [eigenes Repo]
FreeSynergy/Node.Store     ← Plugin-Registry für Node (Daten, kein Code)           [eigenes Repo]
FreeSynergy/Wiki.Store     ← Plugin-Registry für Wiki.rs (zukünftig)               [eigenes Repo]
FreeSynergy/Decidim.Store  ← Plugin-Registry für Decidim.rs (zukünftig)            [eigenes Repo]
```

### FreeSynergy/Lib — Bibliotheken

Alle Präfixe sind `fs-` (FreeSynergy Network).

```
fs-types/              Shared Types (Resource, Meta, TypeSystem, Capability)
fs-error/              Fehlerbehandlung + Auto-Repair + Repairable-Trait
fs-config/             TOML laden/speichern mit Validierung + Auto-Repair
fs-i18n/               Fluent-basierte Schnipsel (actions, nouns, status, errors, ...)
fs-sync/               CRDT-Sync (Automerge-Wrapper)
fs-store/              Universeller Store-Client (Download, Registry, Suche)
fs-plugin-sdk/         WASM Plugin SDK (Traits, wit-bindgen Interfaces)
fs-plugin-runtime/     WASM Host (wasmtime)
fs-federation/         OIDC + SCIM + ActivityPub + WebFinger
fs-auth/               OAuth2 + JWT + Permissions
fs-bridge-sdk/         Bridge-Interface-Traits
fs-container/          Container-Abstraktion (Podman via bollard)
fs-template/           Tera-Wrapper
fs-health/             Health-Check Framework
fs-crypto/             age-Encryption, mTLS, Key-Management
fs-db/                 Datenbank-Abstraktion (SeaORM + rusqlite)
fs-theme/              Theme-System (CSS-Variablen, TUI-Farben)
fs-help/               Kontextsensitives Hilfe-System
```

### FreeSynergy/Node — Deployment-Engine

```
cli/crates/
  fs-node-core/        Node-spezifische Logik + Datentypen (Config, State, Health, Store)
  fs-deploy/           Quadlet-Generation, Zentinel, Reconciliation, Hooks
  fs-dns/              DNS-Provider Integrationen (Hetzner, Cloudflare)
  fs-host/             Host-Management, SSH, Remote-Install, Provisioning
  fs-wizard/           Container-Assistent (Docker Compose → FSN-Modul)
  fs-node-cli/         CLI Binary (clap) — `fsn` Kommando, kein UI-Code
  fs-installer/        Server-Setup-Tooling (Erstinstallation)
```

**Kein UI-Code in Node.** Das UI gehört in Desktop.

### FreeSynergy/Desktop — Desktop-Umgebung

```
crates/
  fs-shell/            Desktop Shell (Taskbar, Window Manager, Wallpaper)
  fs-conductor/        Container/Service/Bot Management (vormals "Admin")
  fs-store/            Package Manager (Browser, Install, Updates)
  fs-studio/           Plugin/Modul/Sprachdatei-Ersteller (+AI optional)
  fs-settings/         System Settings
  fs-profile/          User Profile
  fs-app/              App-Launcher Binary (startet alles)
```

Jedes `fs-*` Crate kann als **eigenständiges Fenster oder Prozess** gestartet werden (Dioxus Multiwindow). `fs-app` ist das Einstiegsprogramm das die Shell lädt.

---

## 3. Datenbank-Empfehlung: SeaORM + rusqlite

### Warum SeaORM?

Nach Analyse aller Optionen ist **SeaORM 2.0 mit rusqlite-Backend** die beste Wahl:

- **Async + Sync**: SeaORM 2.0 hat ein offizielles `sea-orm-sync` Crate mit rusqlite-Backend — perfekt für CLI-Tools wo async Overkill wäre, und das async-Backend für den Server/UI
- **Entity-First Workflow**: Entities definieren → Schema generiert. Passt zu unserem OOP-Ansatz
- **Migrationen eingebaut**: `sea-orm-cli` für Schema-Migrations
- **Multi-DB-fähig**: Startet mit SQLite, kann auf Postgres wechseln wenn nötig (Wiki.rs wird Postgres brauchen)
- **Admin Panel**: SeaORM Pro bietet gratis RBAC-Admin-Panel
- **Wiederverwendbar**: Dieselbe `fs-db` Library kann in Node (SQLite), Wiki.rs (Postgres) und Decidim.rs (Postgres) eingesetzt werden

### Write-Buffering Engine (wie ownCloud)

Für das Problem mit vielen gleichzeitigen Schreibzugriffen (das Du von ownCloud kennst):

```rust
/// fs-db: Write-Buffer für SQLite
pub struct WriteBuffer {
    queue: Vec<BufferedWrite>,
    flush_interval: Duration,    // z.B. 100ms
    max_batch_size: usize,       // z.B. 500 Operationen
    db: DatabaseConnection,
}

impl WriteBuffer {
    /// Schreibt nicht sofort, sondern puffert
    pub async fn enqueue(&mut self, write: BufferedWrite) -> Result<()>;

    /// Flush: Schreibt alle gepufferten Operationen in einer Transaktion
    pub async fn flush(&mut self) -> Result<FlushResult>;

    /// Automatischer Flush per Timer oder Batch-Größe
    pub async fn run_auto_flush(&mut self);
}
```

Das kombiniert SQLite-Vorteile (embedded, keine Infra) mit Batch-Writes (keine Lock-Contention bei vielen Zugriffen).

### Schema in fs-db (wiederverwendbar)

```rust
// fs-db bietet Basis-Entities die jedes Projekt erweitern kann
pub mod entities {
    pub mod resource;     // Basis-Resource mit Metadaten
    pub mod permission;   // RBAC-Permissions
    pub mod sync_state;   // CRDT-Sync-Zustand
    pub mod plugin;       // Installierte Plugins
    pub mod audit_log;    // Audit-Trail
}

// Node erweitert mit eigenen Entities
pub mod node_entities {
    pub mod host;
    pub mod project;
    pub mod module;
    pub mod container;
}
```

---

## 4. Desktop-Architektur (FreeSynergy.Desktop)

### 4.1 Übersicht

FreeSynergy.Desktop ist ein **eigenes Programm und eigenes Repository**. Es ist wie ein echter Desktop (KDE-ähnlich), gebaut mit Dioxus Multiwindow. Jede App (`fs-*`) läuft als eigenständiges Fenster — auf dem Desktop parallel zu anderen, im Web als Tab, im TUI als Panel.

```
┌─────────────────────────────────────────────────────────────────┐
│ FreeSynergy                         [Wallpaper / Hintergrund]  │
│                                                                  │
│  ┌──────────────┐  ┌──────────────────────────────────────┐    │
│  │  Conductor   │  │  Store                               │    │
│  │              │  │                                      │    │
│  │  [Container] │  │  [Paket suchen...]                   │    │
│  │  [Bots]      │  │  ┌────┐ fs-nginx    [Installieren] │    │
│  │  [Ressourcen]│  │  └────┘ fs-postgres [Installieren] │    │
│  └──────────────┘  └──────────────────────────────────────┘    │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│ [⚙ Apps] [Conductor] [Store] [Studio] [Settings]  🔔 12:34 DE │
└─────────────────────────────────────────────────────────────────┘
```

### 4.2 Taskbar (fs-shell)

Wie KDE Plasma — immer sichtbar, konfigurierbar:

- **App-Launcher**: Grid aller installierten Apps + Suche (Win-Taste / Klick)
- **Laufende Apps**: Icons mit Fenster-Vorschau bei Hover (wie KDE)
- **System Tray**: Sync-Status, Netzwerk, Notifications
- **Sprachanzeige**: aktive Sprache, wechselbar per Klick
- **Uhr + Datum**

### 4.3 Apps (fs-*)

| App | Zweck | Standalone? |
|---|---|---|
| `fs-shell` | Taskbar, Window Manager, Wallpaper | Nein (läuft immer) |
| `fs-conductor` | Container/Service/Bot Management | Ja |
| `fs-store` | Package Manager | Ja |
| `fs-studio` | Plugin/Modul/i18n-Ersteller (+AI) | Ja |
| `fs-settings` | System Settings | Ja |
| `fs-profile` | User Profile | Ja |

Standalone = kann auch ohne Shell als eigenes Fenster/Prozess gestartet werden.

### 4.4 Conductor (vormals "Admin")

**Conductor** dirigiert Container, Services und Bots — wie ein Orchesterdirigent:

- Container installieren, starten, stoppen, neustarten
- Ressourcen konfigurieren (CPU, RAM, Volumes, Netzwerk)
- Bots laden und steuern
- Logs und Status in Echtzeit
- Service-Abhängigkeiten visualisieren
- Health-Status aller laufenden Services

```
fsn conductor start <service>     ← CLI
fsn conductor                      ← TUI
fs-conductor                      ← GUI
```

### 4.5 Store (Package Manager)

**Trennung der Verantwortung:**
- **Store** = Discovery + Download + Abhängigkeiten auflösen + Updates + Entfernen
- Bei Installation: **Setup-Wizard** aus Paket-Metadaten (Konfiguration VOR dem ersten Start)
- **Conductor** = Laufzeit-Management (Starten, Stoppen, Ressourcen, Logs)

Wie `apt install` + interaktiver Konfig-Dialog → dann läuft's → Management in Conductor.

```
fsn store search <query>           ← CLI
fsn store install <package>        ← CLI (triggert Setup-Wizard)
fsn store update                   ← CLI
fs-store                          ← GUI
```

### 4.6 Studio (Plugin/Modul/i18n-Ersteller)

Studio ist das Werkzeug um Inhalte für das FSN-Ökosystem zu erstellen:

- **Module Builder**: YAML/Docker-Compose → FSN-Modul (= heutiger Wizard, aus Node extrahiert)
- **Plugin Builder**: WASM-Plugin generieren (wit-bindgen Templates)
- **i18n Editor**: Sprachdateien visuell bearbeiten und erstellen
- **AI-Erweiterung** (optional): Natürlichsprachliche Beschreibung → Modul-Metadaten generiert

```
fsn studio                         ← TUI
fs-studio                         ← GUI
```

### 4.7 Settings

| Bereich | Inhalt |
|---|---|
| **Appearance** | Wallpaper (URL oder Datei-Upload), CSS-Datei (URL oder Upload), Logo, Theme, Dark/Light |
| **Language** | Sprache wählen, Sprachdateien aus Store laden |
| **Service Roles** | Welcher Container für welche Funktion (Auth, Mail, Storage, Git, …) |
| **Accounts** | Verbundene OIDC-Accounts |
| **Desktop** | Taskbar-Position, Autostart-Apps |

### 4.8 Service Roles (erweiterter MIME-Standard)

Wie MIME, aber für **Funktionen** statt Dateitypen. Container registrieren welche Rollen sie erfüllen können. Settings wählt den aktiven Handler pro Rolle.

```toml
[service-roles]
auth     = "kanidm"       # Welcher Container ist Auth-Provider?
mail     = "stalwart"     # Mail-Handler
git      = "forgejo"      # Git-Handler
storage  = "seaweedfs"    # Storage-Handler
wiki     = "outline"      # Wiki-Handler
chat     = "tuwunel"      # Chat-Handler
tasks    = "vikunja"      # Task-Handler
```

Container-Metadaten deklarieren welche Rollen sie unterstützen:

```toml
[module.roles]
provides = ["auth", "iam"]   # Diese Rollen kann dieser Container übernehmen
requires = ["mail"]          # Diese Rollen müssen erfüllt sein
```

---

## 5. UI-Architektur (Fenster-System)

### 5.1 Alle Einblendungen sind Fenster

Konsistentes Verhalten überall:

```rust
pub struct Window {
    pub id: WindowId,
    pub title: LocalizedString,
    pub content: Box<dyn WindowContent>,
    pub closable: bool,             // Immer true
    pub buttons: Vec<WindowButton>, // OK, Cancel, Apply
    pub size: WindowSize,
    pub scrollable: bool,           // Automatisch wenn Inhalt > Fenster
    pub help_topic: Option<String>, // Für kontextsensitive Hilfe
}

pub enum WindowButton {
    Ok,          // Bestätigen + Schließen
    Cancel,      // Abbrechen + Schließen
    Apply,       // Übernehmen (bleibt offen)
    Custom { label_key: String, action: WindowAction },
}
```

### 5.2 Container-Render-Modi (Metadaten pro Modul)

```toml
[module.ui]
supports_web      = true   # Hat Web-Interface
supports_tui      = false  # Hat TUI-Interface (selten)
supports_desktop  = true   # Kann als Desktop-App eingebettet werden
supports_api_only = true   # Nur API, kein UI

open_mode         = "iframe"   # "iframe" | "external_browser" | "embedded" | "api"
web_url_template  = "https://{{ domain }}/{{ service_path }}"
```

### 5.3 Scrolling (auch in TUI)

Jedes Formular und jede Liste ist **automatisch scrollbar** wenn der Inhalt nicht passt:

```rust
pub trait Scrollable {
    fn content_height(&self) -> u32;
    fn viewport_height(&self) -> u32;
    fn scroll_offset(&self) -> u32;
    fn needs_scroll(&self) -> bool {
        self.content_height() > self.viewport_height()
    }
}
```

Maus-Scrolling + Tastatur (PgUp/PgDn/Home/End) in allen Interfaces.

### 5.4 Hilfe-System (fs-help)

```rust
pub struct HelpSystem {
    topics: HashMap<String, HelpTopic>,
    i18n: I18n,
}

pub struct HelpTopic {
    pub id: String,
    pub title_key: String,       // i18n-Key
    pub content_key: String,     // i18n-Key
    pub related: Vec<String>,    // Verwandte Themen
    pub context: HelpContext,    // Wo diese Hilfe angezeigt wird
}

impl HelpSystem {
    /// Kontextsensitive Hilfe: Was ist gerade aktiv?
    pub fn help_for_context(&self, ctx: &str) -> Option<&HelpTopic>;

    /// Suche in Hilfetexten
    pub fn search(&self, query: &str) -> Vec<&HelpTopic>;

    /// Anzeigen als Fenster
    pub fn show_help_window(&self, topic: &str) -> Window;
}
```

Aufruf: **F1** (Desktop/Web), **?** (TUI), Menü, oder Hilfe-Button in jedem Fenster.

---

## 6. Theme-System (fs-theme)

### 6.1 Eine Datei regiert alles

Der Benutzer (oder eine KI die die Website baut) liefert **eine einzige Theme-Datei** ab. Diese wird für Dioxus (Desktop/Web) UND TUI interpretiert.

### 6.2 Theme-Format: `theme.toml`

```toml
[theme]
name    = "FreeSynergy Default"
version = "1.0.0"
author  = "KalEl"

[colors]
primary        = "#2563eb"
primary_hover  = "#1d4ed8"
primary_text   = "#ffffff"

secondary      = "#64748b"
secondary_hover = "#475569"
secondary_text = "#ffffff"

bg_base    = "#ffffff"
bg_surface = "#f8fafc"
bg_overlay = "#f1f5f9"
bg_sidebar = "#1e293b"

text_primary   = "#0f172a"
text_secondary = "#475569"
text_muted     = "#94a3b8"
text_inverse   = "#ffffff"

success = "#22c55e"
warning = "#f59e0b"
error   = "#ef4444"
info    = "#3b82f6"

border_default = "#e2e8f0"
border_focus   = "#2563eb"

[typography]
font_family   = "Inter, system-ui, sans-serif"
font_mono     = "JetBrains Mono, monospace"
font_size_base = "16px"
font_size_sm  = "14px"
font_size_lg  = "20px"
font_size_xl  = "24px"
font_size_2xl = "30px"
line_height   = "1.5"

[spacing]
unit      = "4px"
radius_sm = "4px"
radius_md = "8px"
radius_lg = "12px"

[tui]
# Wird automatisch aus [colors] abgeleitet, kann überschrieben werden
primary_fg   = "blue"
primary_bg   = "default"
sidebar_fg   = "white"
sidebar_bg   = "dark_gray"
border_style = "rounded"    # "plain" | "rounded" | "double" | "thick"
status_ok    = "green"
status_error = "red"
status_warn  = "yellow"
```

### 6.3 CSS-Variablen Konvention (für Website-KI)

```
Datei: theme.css

Variablen-Namensschema (Präfix IMMER --fs-):
  --fs-color-primary: #2563eb;
  --fs-color-primary-hover: #1d4ed8;
  --fs-color-bg-base: #ffffff;
  --fs-color-bg-surface: #f8fafc;
  --fs-color-text-primary: #0f172a;
  --fs-color-success: #22c55e;
  --fs-color-warning: #f59e0b;
  --fs-color-error: #ef4444;
  --fs-font-family: 'Inter', system-ui, sans-serif;
  --fs-font-mono: 'JetBrains Mono', monospace;
  --fs-font-size-base: 16px;
  --fs-spacing-unit: 4px;
  --fs-radius-md: 8px;

Liefere NUR :root { ... } — kein Layout, keine Komponenten.
FreeSynergy.Node konvertiert diese automatisch in theme.toml.
```

### 6.4 Konvertierung

```rust
/// fs-theme: Konvertiert zwischen Formaten
pub struct ThemeEngine {
    theme: Theme,
}

impl ThemeEngine {
    pub fn from_toml(path: &Path) -> Result<Self>;
    pub fn from_css(path: &Path) -> Result<Self>;  // CSS Custom Properties → Theme
    pub fn to_css(&self) -> String;                 // → Dioxus Web
    pub fn to_tui_palette(&self) -> TuiPalette;    // → TUI
    pub fn to_tailwind_config(&self) -> String;    // → Tailwind
}
```

### 6.5 Mehrere Themes, wechselbar

Themes werden wie Plugins über den Store verteilbar und in Settings wechselbar:

```toml
[appearance]
active_theme     = "freesynergy-default"
available_themes = ["freesynergy-default", "freesynergy-dark", "helfa-green"]
```

---

## 7. i18n — Schnipsel-System

### Kleine, wiederverwendbare Bausteine

```
locales/{lang}/
  actions.ftl     → save, delete, edit, search, confirm, cancel, ...
  nouns.ftl       → module, server, project, host, plugin, store, ...
  status.ftl      → online, offline, error, loading, syncing, ...
  errors.ftl      → file-not-found, invalid-config, connection-failed, ...
  phrases.ftl     → select-item, confirm-delete, welcome-to, ...
  time.ftl        → ago, minutes, hours, days, just-now, ...
  validation.ftl  → required-field, invalid-email, too-short, ...
  help.ftl        → help-dashboard, help-wizard, help-store, ...
```

Zusammengesetzt im Code:
```rust
// t("action-save") → "Save" / "Speichern"
// t_phrase("phrase-confirm-delete", [("item", t("noun-module"))])
//   → "Delete module?" / "Modul löschen?"
```

---

## 8. Error-Handling + Auto-Repair

Siehe Plan v2 — unverändert. Zusammenfassung:
- **Repairable-Trait** auf allen Konfig-Typen
- **AutoRepaired** → Toast-Notification
- **NeedsUserDecision** → Dialog mit Optionen
- **Unrecoverable** → Fehler anzeigen, nicht öffnen
- Backup immer anbieten bevor repariert wird

---

## 9. Container-Assistent (fs-wizard / fs-studio)

Der Container-Assistent lebt in **fs-studio** (GUI) und **fs-wizard** (Library):
- YAML/Docker-Compose eingeben (Text, URL, Datei)
- Automatische Typ-Erkennung (Image-Name, Ports, Volumes)
- Modul-Generation mit Standard-Werten
- Erklärungen was fehlt (APIs, Abhängigkeiten)
- Benutzer wählt Purpose/Service-Role
- Optional: AI-gestützte Generierung

---

## 10. Typ-System + Schnittstellen

Siehe Plan v2 — unverändert. Zusammenfassung:
- **Capability-Trait**: Was kann ein Service? (APIs, Events, Formate)
- **Requirement-Trait**: Was braucht ein Service?
- **TypeRegistry**: Validiert Abhängigkeiten, findet kompatible Bridges
- Pro Typ: TOML-Definition im Store mit APIs, Events, Bridge-Kompatibilität
- **Service Roles** (siehe 4.8): Extended MIME für Funktionen

---

## 11. CRDT + Sync + Federation + Store + Bridges + Permissions

Alle Details aus Plan v2 bleiben bestehen. Hier nur die Entscheidungen:

| Thema | Entscheidung |
|---|---|
| CRDT | **Automerge** (3 wenn stabil, sonst 0.5 stable). Beitrag zum Projekt möglich. |
| Plugin-Interface | **wit-bindgen** (WASM Component Model Standard) |
| ActivityPub | **activitypub_federation** (Lemmy, Axum-kompatibel) |
| Datenbank | **SeaORM 2.0** + rusqlite (sync) + sqlx (async/Postgres) |
| Templates | **Tera** (einziger Template-Engine) |

---

## 12. Verbesserungsvorschläge

### 12.1 Versionierung & Changelog

Jede `fs-*` Library bekommt **eigene SemVer-Versionierung**. CHANGELOG.md pro Crate, nicht nur global. Nutze `cargo-release` für koordinierte Releases.

### 12.2 Feature Flags überall

Jede Library sollte granulare Feature-Flags haben:

```toml
[features]
default      = ["sqlite"]
sqlite       = ["sea-orm/rusqlite"]
postgres     = ["sea-orm/sqlx-postgres"]
sync         = ["automerge"]
federation   = ["activitypub_federation", "openidconnect"]
wasm-plugins = ["wasmtime"]
```

Das hält die Compile-Zeiten kurz. Wiki.rs braucht vielleicht `federation` + `postgres` aber kein `wasm-plugins`.

### 12.3 CI/CD von Anfang an

- **GitHub Actions**: Build, Test, Clippy, Rustfmt auf jedem Push
- **cargo-deny**: License-Check, Advisory-DB-Check
- **Dependabot**: Automatische Dependency-Updates
- **Nightly Fuzzing**: cargo-fuzz auf fs-config, fs-sync, fs-template (alles was User-Input parst)

### 12.4 Dokumentation

- **Jede fs-* Crate**: README.md + `#[doc]` auf allen pub Items
- **docs.rs** automatisch (bei Publish auf crates.io)
- **Architektur-Docs**: `docs/ARCHITECTURE.md` pro Repo
- **Beispiele**: `examples/` Verzeichnis in jeder Library

### 12.5 Error-Recovery für Netzwerk

```rust
pub struct RetryPolicy {
    pub max_retries: u32,
    pub backoff: BackoffStrategy,
    pub on_failure: FailureAction,  // Cache nutzen, Offline-Modus, Benutzer fragen
}
```

Wenn der Store nicht erreichbar ist → lokalen Cache nutzen. Wenn ein Host offline ist → markieren, nicht crashen.

### 12.6 Offline-First

Store-Katalog wird gecacht, Konfigurationen sind lokal, CRDT-Sync passiert wenn Verbindung da ist. Kein Feature darf eine Netzwerkverbindung voraussetzen außer explizit netzwerk-basierten Aktionen.

### 12.7 Audit-Log

```rust
pub struct AuditEntry {
    pub timestamp: DateTime<Utc>,
    pub actor: Subject,
    pub action: AuditAction,
    pub target: ResourceRef,
    pub details: Value,
    pub source_host: HostRef,
}
```

Wird per CRDT synchronisiert → verteiltes, konsistentes Audit-Log.

### 12.8 Graceful Degradation

Wenn ein Plugin nicht lädt → Rest funktioniert trotzdem. Wenn CRDT-Sync fehlschlägt → lokaler Zustand bleibt nutzbar. Wenn ein Host offline ist → andere Hosts arbeiten weiter.

### 12.9 Config-Schema als JSON-Schema

Plugin-Metadaten und Modul-Konfigurationen bringen ein **JSON-Schema** mit (Format bleibt TOML):
- Automatische Validierung
- UI-Generierung (Forms aus Schema generieren)
- Dokumentation (Schema → Docs)

### 12.10 Migration von v1

- Modul-Definitionen → migrieren in Node.Store Format
- Deployment-Logik → migrieren in fs-deploy
- i18n-Strings → migrieren in fs-i18n Schnipsel-Format
- Ein `migration/` Verzeichnis mit Skripten die alte Configs konvertieren

### 12.11 ratatui/rat-salsa entfernen

Da wir auf **Dioxus** umsteigen (hat `dioxus-terminal` für TUI), fällt ratatui komplett weg:
1. Alle bisherigen TUI-Nodes neu als Dioxus-Komponenten in `fs-*`
2. `rat-widget`, `ratatui`, `rat-salsa` aus allen `Cargo.toml` entfernen
3. `fs-tui` Crate wird aufgelöst — Komponenten wandern in `fs-shell` oder `fs-*` Libraries

---

## 13. Umsetzungsplan

### Phase 0: Setup ✓

- [x] `FreeSynergy/Lib` erstellen, CI einrichten
- [x] `FreeSynergy/Desktop` erstellen, CI einrichten
- [x] `FreeSynergy/UI` archivieren
- [x] `FreeSynergy/Node` bereinigen (ratatui/rat-salsa entfernen, fs-app entfernen)
- [x] CLAUDE.md in allen Repos aktualisieren

### Phase 1: Fundament (FreeSynergy.Lib) ✓

fs-types, fs-error, fs-config, fs-i18n, fs-theme, fs-help, fs-db, fs-health

### Phase 2: CRDT + Sync (Stub)

fs-sync (Automerge) — Stub implementiert, aktive Integration ausstehend

### Phase 3: Store + Plugins (Stub)

fs-store, fs-plugin-sdk, fs-plugin-runtime — Stubs implementiert

### Phase 4: Auth + Federation (Stub)

fs-auth, fs-federation, fs-crypto — Stubs implementiert

### Phase 5: Container + Templates (Stub)

fs-container, fs-template, fs-health — implementiert

### Phase 6: Node Application ✓

fs-node-core, fs-deploy, fs-host, fs-wizard, fs-node-cli, fs-dns, fs-installer

### Phase 7: Desktop (FreeSynergy.Desktop)

fs-shell, fs-conductor, fs-store, fs-settings, fs-profile, fs-studio, fs-app — in Planung

### Phase 8: Bridges (ongoing)

fs-bridge-sdk + erste WASM-Bridge-Plugins

---

## 14. Vollständiger Bibliotheken-Stack

### Kern

| Crate | Version | Zweck |
|---|---|---|
| `dioxus` | 0.7.x | UI: TUI + Desktop + Web + Mobile |
| `serde` + `toml` + `serde_json` | 1 / 0.8 / 1 | Serialisierung |
| `sea-orm` | 2.0 | ORM (async: sqlx, sync: rusqlite) |
| `sea-orm-sync` | 2.0 | Sync SQLite für CLI |
| `automerge` | 0.5+ / 3.x | CRDT |
| `tera` | 1 | Templates |
| `fluent` | 0.16 | i18n |
| `activitypub_federation` | 0.7 | ActivityPub |

### Netzwerk

| Crate | Zweck |
|---|---|
| `reqwest` (rustls) | HTTP-Client |
| `axum` (via Dioxus) | HTTP-Server |
| `tokio-tungstenite` | WebSocket |
| `russh` | SSH |
| `rustls` + `rcgen` | TLS + Zertifikate |
| `tonic` | gRPC |

### Auth

| Crate | Zweck |
|---|---|
| `openidconnect` | OIDC |
| `oauth2` | OAuth2 |
| `jsonwebtoken` | JWT |
| `age` | Secrets |

### Plugins

| Crate | Zweck |
|---|---|
| `wasmtime` + `wasmtime-wasi` | WASM Runtime (Standard) |
| `wit-bindgen` | Component Model Interfaces |
| `libloading` + `abi_stable` | Native (nur Ausnahmen) |

### Container

| Crate | Zweck |
|---|---|
| `bollard` | Podman/Docker API |
| `serde_yaml` | YAML-Parse |
| `tokio-cron-scheduler` | Scheduling |
| `backon` | Retry mit Backoff |

### Qualität

| Crate | Zweck |
|---|---|
| `thiserror` + `anyhow` | Errors |
| `tracing` + `tracing-subscriber` | Logging |
| `opentelemetry` + `opentelemetry-otlp` | Observability |
| `rstest` + `insta` + `mockall` | Testing |
| `cargo-fuzz` | Fuzzing |
| `testcontainers` | Integration Tests |
| `schemars` | JSON-Schema Generation |
| `cargo-deny` | License/Advisory Check |

---

## 15. Zusammenfassung aller Entscheidungen

| Frage | Entscheidung |
|---|---|
| UI-Framework | **Dioxus 0.7.x** (TUI + Desktop + Web + Mobile) |
| Datenbank | **SeaORM 2.0** (rusqlite sync + sqlx async) |
| CRDT | **Automerge** (von Tag 1) |
| Plugins | **WASM-First** (wit-bindgen, wasmtime) |
| Templates | **Nur Tera** |
| Federation | **OIDC + SCIM + ActivityPub** (von Tag 1) |
| ActivityPub Crate | **activitypub_federation** |
| Theme-System | **Eine Datei** (theme.toml oder theme.css → konvertierbar) |
| CSS-Präfix | **--fs-** (nicht --fsy-) |
| Fenster | **Alle Einblendungen sind Fenster** (OK/Cancel/Apply) |
| Hilfe | **Immer aufrufbar** (F1, ?, Menü) |
| Scrolling | **Automatisch** wenn Inhalt > Viewport |
| TUI-Framework | **Dioxus terminal** (ratatui/rat-salsa entfernt) |
| Desktop | **Eigenes Repo** (FreeSynergy/Desktop) |
| Admin-Begriff | **Conductor** (Container/Service/Bot Management) |
| Wizard | **fs-studio** (GUI) + **fs-wizard** (Library) |
| MIME-Erweiterung | **Service Roles** (fs-Prefix in TOML) |
| Package Manager | **fs-store** (Discovery+Install+Wizard), Conductor für Laufzeit |
| Crate-Präfix Lib | **fs-** (nicht fsy-) |
| Crate-Präfix Desktop | **fs-** |
| Sprache im Code | **Englisch** |
| Sprache hier | **Deutsch** |
| Lib-Veröffentlichung | **crates.io** (wenn APIs stabil) |
| Repo-Struktur | **Lib + Node + Desktop + Store-Repos** (je eigenes Repo) |
| Wiki.rs/Decidim.rs | **Demnächst** — fs-* Libraries müssen stabil sein |

---

## Nächster Schritt

Phase 7: FreeSynergy.Desktop — Dioxus-App mit fs-shell, fs-conductor (Hosts/Services/Projekte), fs-store (Plugin-Browser), fs-studio (Modul-Builder), fs-settings, fs-profile, fs-app.

Parallel: Phase 2–4 Stubs in FreeSynergy.Lib aktivieren (fs-sync Automerge, fs-auth OIDC, fs-federation ActivityPub).
