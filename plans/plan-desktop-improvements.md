# FreeSynergy.Desktop — Arbeitsplan für Claude Code

**Repo:** https://github.com/FreeSynergy/FreeSynergy.Desktop
**Regeln:** English in code/comments. Jede Änderung mit `cargo check`. Stubs durch echten Code ersetzen. Alten/ungenutzten Code löschen.

---

## VORHER: Analyse

Bevor Du IRGENDETWAS änderst, führe diese Befehle aus und zeige die Ergebnisse:

```bash
find . -name "*.rs" | sort
cat Cargo.toml
for f in $(find crates -name "Cargo.toml" -maxdepth 3); do echo "=== $f ==="; head -20 "$f"; done
grep -rn "Fsy\|fsy_\|fsy-" --include="*.rs" --include="*.toml" . | head -40
grep -rn "podman.sock\|bollard\|docker" --include="*.rs" . | head -20
grep -rn "todo!\|unimplemented!\|stub\|STUB\|placeholder\|PLACEHOLDER" --include="*.rs" . | head -30
grep -rn "color\|background.*#\|Color::\|rgb(" --include="*.rs" . | head -30
```

Erstelle eine Liste aller Probleme BEVOR du anfängst.

---

## Aufgabe 1: Repository umbenennen

Das Repo heißt `FreeSynergy.Desktop` — der GitHub-Account heißt schon `FreeSynergy`, also ist der Projektname doppelt.

**Aktion:** Ändere ALLE internen Referenzen:
- In Cargo.toml: `name = "freesynergy-desktop"` o.ä. → `name = "fsn-desktop"` (oder wie es schon heißt, Hauptsache konsistent `fsn-`)
- In README, CLAUDE.md, Kommentaren: "FreeSynergy.Desktop" bleibt als Projektname, aber das Repo sollte intern konsistent `fsn-desktop-*` Crates haben
- Prüfe ob in Cargo.toml noch `fsy-` oder `Fsy` Referenzen stehen → alles `fsn-`

**Hinweis an den Menschen:** Das Repo auf GitHub umbenennen (Settings → Rename) von `FreeSynergy.Desktop` auf `Desktop` ist ein manueller Schritt, den Claude Code nicht machen kann.

---

## Aufgabe 2: Dark + Light Theme

### 2.1 Dark Theme ("Midnight Blue")

Suche die Stelle wo das Theme/Farben definiert werden (CSS-String, Konstanten, oder Theme-Struct) und ersetze mit:

```rust
pub const DARK_THEME: &str = r#"
:root, [data-theme="dark"] {
    --fsn-bg-base: #0c1222;
    --fsn-bg-surface: #162032;
    --fsn-bg-elevated: #1e2d45;
    --fsn-bg-sidebar: #0a0f1a;
    --fsn-bg-card: #1a2538;
    --fsn-bg-input: #0f1a2e;
    --fsn-bg-hover: #243352;

    --fsn-text-primary: #e8edf5;
    --fsn-text-secondary: #a0b0c8;
    --fsn-text-muted: #5a6e88;
    --fsn-text-bright: #ffffff;

    --fsn-primary: #4d8bf5;
    --fsn-primary-hover: #3a78e8;
    --fsn-primary-text: #ffffff;
    --fsn-primary-glow: rgba(77, 139, 245, 0.35);

    --fsn-accent: #22d3ee;

    --fsn-success: #34d399;
    --fsn-warning: #fbbf24;
    --fsn-error: #f87171;
    --fsn-info: #60a5fa;

    --fsn-border: rgba(148, 170, 200, 0.18);
    --fsn-border-focus: #4d8bf5;

    --fsn-sidebar-bg: #0a0f1a;
    --fsn-sidebar-text: #a0b0c8;
    --fsn-sidebar-active: #4d8bf5;
    --fsn-sidebar-active-bg: rgba(77, 139, 245, 0.15);
    --fsn-sidebar-hover: rgba(255, 255, 255, 0.05);

    --fsn-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
    --fsn-radius: 10px;
    --fsn-transition: all 180ms ease;
}
"#;
```

### 2.2 Light Theme ("Cloud White")

```rust
pub const LIGHT_THEME: &str = r#"
[data-theme="light"] {
    --fsn-bg-base: #f8fafc;
    --fsn-bg-surface: #ffffff;
    --fsn-bg-elevated: #f1f5f9;
    --fsn-bg-sidebar: #1e293b;
    --fsn-bg-card: #ffffff;
    --fsn-bg-input: #f1f5f9;
    --fsn-bg-hover: #e2e8f0;

    --fsn-text-primary: #0f172a;
    --fsn-text-secondary: #475569;
    --fsn-text-muted: #94a3b8;
    --fsn-text-bright: #0f172a;

    --fsn-primary: #2563eb;
    --fsn-primary-hover: #1d4ed8;
    --fsn-primary-text: #ffffff;
    --fsn-primary-glow: rgba(37, 99, 235, 0.2);

    --fsn-accent: #0891b2;

    --fsn-success: #16a34a;
    --fsn-warning: #d97706;
    --fsn-error: #dc2626;
    --fsn-info: #2563eb;

    --fsn-border: #e2e8f0;
    --fsn-border-focus: #2563eb;

    --fsn-sidebar-bg: #1e293b;
    --fsn-sidebar-text: #cbd5e1;
    --fsn-sidebar-active: #60a5fa;
    --fsn-sidebar-active-bg: rgba(96, 165, 250, 0.15);
    --fsn-sidebar-hover: rgba(255, 255, 255, 0.08);

    --fsn-shadow: 0 1px 8px rgba(0, 0, 0, 0.08);
    --fsn-radius: 10px;
    --fsn-transition: all 180ms ease;
}
"#;
```

### 2.3 Theme-Switching

Alle UI-Elemente MÜSSEN `var(--fsn-*)` verwenden, KEINE hardcodierten Farben. Suche und ersetze JEDE Stelle wo eine Farbe direkt steht (z.B. `background: "#1a1a2e"`) durch die passende CSS-Variable (`background: var(--fsn-bg-base)`).

Das Theme wird gewechselt durch ein `data-theme` Attribut auf dem Root-Element:
```rust
// In der App-Komponente
let theme = use_signal(|| settings.theme.clone()); // "dark" oder "light"
rsx! {
    div { 
        "data-theme": "{theme}",
        // ... rest of app
    }
}
```

### 2.4 CSS-Wiederverwendbarkeit für Wiki.rs

Ja, dieselben CSS-Variablen funktionieren in Wiki.rs. Die Regel:
- Variablen-Prefix ist `--fsn-` (für FreeSynergy-Ökosystem)
- Jedes Projekt (Desktop, Wiki.rs) lädt dieselbe Theme-CSS-Datei
- Projektspezifische Ergänzungen nutzen zusätzliche Variablen (z.B. `--fsn-wiki-*`)

---

## Aufgabe 3: Window-Buttons (macOS → KDE-Style)

Suche die Stelle wo die Window-Buttons (Minimize, Maximize, Close) gerendert werden. Das sind wahrscheinlich drei farbige Kreise (macOS-Style: rot/gelb/grün).

Ersetze durch KDE/Breeze-Style Buttons:

```rust
#[component]
fn WindowControls(on_minimize: EventHandler, on_maximize: EventHandler, on_close: EventHandler) -> Element {
    rsx! {
        div { class: "fsn-window-controls",
            style: "display: flex; gap: 2px; align-items: center;",
            // KDE Breeze style: rectangular buttons with icons
            button {
                class: "fsn-window-btn fsn-window-btn-minimize",
                onclick: move |_| on_minimize.call(()),
                title: "Minimize",
                // Horizontal line icon
                svg { view_box: "0 0 16 16", width: "10", height: "10",
                    line { x1: "3", y1: "8", x2: "13", y2: "8", stroke: "var(--fsn-text-secondary)", stroke_width: "1.5" }
                }
            }
            button {
                class: "fsn-window-btn fsn-window-btn-maximize",
                onclick: move |_| on_maximize.call(()),
                title: "Maximize",
                // Square icon
                svg { view_box: "0 0 16 16", width: "10", height: "10",
                    rect { x: "3", y: "3", width: "10", height: "10", fill: "none", stroke: "var(--fsn-text-secondary)", stroke_width: "1.5" }
                }
            }
            button {
                class: "fsn-window-btn fsn-window-btn-close",
                onclick: move |_| on_close.call(()),
                title: "Close",
                // X icon
                svg { view_box: "0 0 16 16", width: "10", height: "10",
                    line { x1: "4", y1: "4", x2: "12", y2: "12", stroke: "var(--fsn-text-secondary)", stroke_width: "1.5" }
                    line { x1: "12", y1: "4", x2: "4", y2: "12", stroke: "var(--fsn-text-secondary)", stroke_width: "1.5" }
                }
            }
        }
    }
}
```

CSS für die Buttons:
```css
.fsn-window-btn {
    width: 28px; height: 28px;
    border: none; border-radius: 4px;
    background: transparent;
    cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: var(--fsn-transition);
}
.fsn-window-btn:hover { background: var(--fsn-bg-hover); }
.fsn-window-btn-close:hover { background: var(--fsn-error); }
.fsn-window-btn-close:hover svg line { stroke: white; }
```

ALTERNATIV: Wenn `with_decorations(true)` gesetzt ist, rendert das System die Buttons nativ. In dem Fall braucht man keine eigenen. Prüfe was aktuell gesetzt ist.

---

## Aufgabe 4: Store — Alle Pakettypen anzeigen

Der Store zeigt momentan nur Plugins. Er muss auch zeigen: Languages, Themes, Widgets, Bot-Commands, Bridges.

Suche die Store-View und das Datenmodell. Erweitere:

```rust
pub enum StoreCategory {
    Plugin,      // Service-Module (Kanidm, Forgejo, ...)
    Language,    // Sprach-Pakete
    Theme,       // CSS-Themes
    Widget,      // Desktop-Widgets
    BotCommand,  // Bot-Commands
    Bridge,      // Service-Bridges
    All,         // Alle anzeigen
}
```

In der Store-UI: Tab-Leiste oder Filter-Dropdown:

```
[Alle] [Plugins] [Sprachen] [Themes] [Widgets] [Bots] [Bridges]
```

Jede Kategorie hat ein eigenes Icon und eine eigene Farbe/Badge.

---

## Aufgabe 5: Sprach-Dropdown — Nur installierte Sprachen

Suche den Sprach-Dropdown/Select in den Settings oder der Toolbar.

Regeln:
1. Zeige NUR installierte Sprachen
2. Standard: Nur Englisch ist installiert. Weitere kommen über den Store.
3. Wenn die Liste mehr als 8 Einträge hat → Scrollbar (CSS: `max-height: 320px; overflow-y: auto`)
4. Am Ende des Dropdowns: Button "Weitere Sprachen installieren..." der zum Store → Languages führt

```rust
#[component]
fn LanguageSelector(installed_languages: Vec<LangMeta>, current: String) -> Element {
    rsx! {
        select {
            class: "fsn-select fsn-lang-select",
            style: "max-height: 320px; overflow-y: auto;",
            value: "{current}",
            for lang in &installed_languages {
                option { value: "{lang.code}", "{lang.name} ({lang.native_name})" }
            }
        }
        button {
            class: "fsn-btn fsn-btn-ghost fsn-btn-sm",
            onclick: navigate_to_store_languages,
            "＋ Install more languages..."
        }
    }
}
```

---

## Aufgabe 6: Widgets (Uhr, Datum, System-Info)

Erstelle ein neues Modul `widgets/` oder ergänze die Home-View:

### Clock Widget
```rust
#[component]
fn ClockWidget() -> Element {
    let time = use_signal(|| chrono::Local::now());
    // Update every second
    use_effect(move || {
        let interval = gloo_timers::callback::Interval::new(1000, move || {
            time.set(chrono::Local::now());
        });
        move || drop(interval)
    });
    
    rsx! {
        div { class: "fsn-widget fsn-widget-clock",
            div { class: "fsn-widget-time", "{time.read().format(\"%H:%M\")}" }
            div { class: "fsn-widget-date", "{time.read().format(\"%A, %d. %B %Y\")}" }
        }
    }
}
```

CSS:
```css
.fsn-widget-clock {
    text-align: center; padding: 20px;
    background: var(--fsn-bg-card);
    border: 1px solid var(--fsn-border);
    border-radius: var(--fsn-radius);
}
.fsn-widget-time {
    font-size: 3rem; font-weight: 200;
    color: var(--fsn-text-primary);
    font-variant-numeric: tabular-nums;
}
.fsn-widget-date {
    font-size: 0.9rem; color: var(--fsn-text-secondary);
    margin-top: 4px;
}
```

### System Info Widget
```rust
#[component]
fn SystemInfoWidget() -> Element {
    let info = use_signal(|| get_system_info());
    rsx! {
        div { class: "fsn-widget fsn-widget-sysinfo",
            div { class: "fsn-widget-row",
                span { class: "fsn-widget-label", "Hostname" }
                span { class: "fsn-widget-value", "{info.read().hostname}" }
            }
            div { class: "fsn-widget-row",
                span { class: "fsn-widget-label", "Uptime" }
                span { class: "fsn-widget-value", "{info.read().uptime}" }
            }
            div { class: "fsn-widget-row",
                span { class: "fsn-widget-label", "Memory" }
                ProgressBar { value: info.read().memory_used_pct, color: "var(--fsn-primary)" }
            }
            div { class: "fsn-widget-row",
                span { class: "fsn-widget-label", "Disk" }
                ProgressBar { value: info.read().disk_used_pct, color: "var(--fsn-accent)" }
            }
        }
    }
}
```

---

## Aufgabe 7: Disabled Buttons (allgemein)

Definiere EINMAL in der globalen CSS eine Disabled-Regel:

```css
.fsn-btn:disabled, .fsn-btn[aria-disabled="true"] {
    opacity: 0.4;
    cursor: not-allowed;
    pointer-events: none;
}
```

Dann in jedem Button der "nicht installierbar" ist:
```rust
Button {
    label: "Install",
    disabled: !can_install,  // ← Wird automatisch grau
    on_click: install_handler,
}
```

---

## Aufgabe 8: Kanidm-Paket erstellen

Erstelle im Store (oder als lokale Testdatei) ein vollständiges Kanidm-Modul:

### 8.1 Package-Manifest

```toml
[package]
id = "iam/kanidm"
name = "Kanidm"
version = "1.5.0"
type = "container"
purpose = "iam"
description_key = "pkg-kanidm-description"
icon = "kanidm"
icon_url = "https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/svg/kanidm.svg"

[package.metadata]
author = "Kanidm Project"
license = "MPL-2.0"
homepage = "https://kanidm.com"
repository = "https://github.com/kanidm/kanidm"
min_node_version = "0.1.0"

[package.capabilities]
oidc-provider = true
scim-server = true
mfa = true
webauthn = true
multi-tenant = true
radius = true
ldap = false
saml = false

[package.container]
image = "docker.io/kanidm/server:latest"
healthcheck = "CMD /sbin/kanidmd healthcheck"
published_ports = []

[[package.container.volumes]]
source = "data"
target = "/data"
persistent = true

[package.api]
type = "rest"
base_path = "/v1"
auth = "oauth2"

[package.events]
emits = ["user-created", "user-updated", "user-deleted", "login-success", "login-failed", "group-changed"]
listens = ["user-provisioned"]

# --- Variables with priorities and role-types ---

[[variables]]
name = "KANIDM_DOMAIN"
label_key = "var-domain"
type = "hostname"
role = ""
priority = 1
required = true
default = "{{ host.domain }}"
description_key = "var-kanidm-domain-desc"

[[variables]]
name = "KANIDM_ORIGIN"
label_key = "var-origin"
type = "url"
role = ""
priority = 1
required = true
default = "https://auth.{{ host.domain }}"
description_key = "var-kanidm-origin-desc"

[[variables]]
name = "KANIDM_BINDADDRESS"
label_key = "var-bind-address"
type = "string"
role = ""
priority = 2
required = false
default = "0.0.0.0:8443"

[[variables]]
name = "KANIDM_LDAP_BINDADDRESS"
label_key = "var-ldap-bind"
type = "string"
role = ""
priority = 3
required = false
default = ""
description_key = "var-kanidm-ldap-desc"

[[variables]]
name = "KANIDM_DB_FS_TYPE"
label_key = "var-db-fs-type"
type = "select"
options = ["zfs", "other"]
role = ""
priority = 3
required = false
default = "other"

[[variables]]
name = "KANIDM_TLS_CHAIN"
label_key = "var-tls-chain"
type = "path"
role = ""
priority = 2
required = true
default = "/data/chain.pem"

[[variables]]
name = "KANIDM_TLS_KEY"
label_key = "var-tls-key"
type = "private-key"
role = ""
priority = 2
required = true
default = "/data/key.pem"

[[variables]]
name = "KANIDM_BACKUP_SCHEDULE"
label_key = "var-backup-schedule"
type = "cron"
role = ""
priority = 2
default = "00 22 * * *"

[[variables]]
name = "KANIDM_BACKUP_VERSIONS"
label_key = "var-backup-versions"
type = "integer"
role = ""
priority = 3
default = "7"
```

### 8.2 Store Detail-View

Wenn man ein Paket im Store anklickt, öffnet sich rechts ein Detail-Panel:

```
┌─ Store ─────────────┬─ Kanidm ──────────────────────────┐
│                      │                                    │
│ [Suche...]           │  [ICON: kanidm.svg]               │
│                      │                                    │
│ ☑ Kanidm       ←──  │  Kanidm v1.5.0                    │
│ ☐ KeyCloak           │  Identity & Access Management      │
│ ☐ Forgejo            │                                    │
│ ☐ Outline            │  Capabilities:                     │
│ ☐ Stalwart           │  OIDC ✓  SCIM ✓  MFA ✓           │
│                      │  WebAuthn ✓  Multi-Tenant ✓       │
│                      │  RADIUS ✓  LDAP ✗  SAML ✗        │
│                      │                                    │
│                      │  Requires: (none)                  │
│                      │  Image: kanidm/server:latest       │
│                      │                                    │
│                      │  [Install]  [More Info]            │
│                      │                                    │
└──────────────────────┴────────────────────────────────────┘
```

---

## Aufgabe 9: Allgemeines Aufräumen

### 9.1 Alle Stubs durch echten Code ersetzen

Suche alle `todo!()`, `unimplemented!()`, und Placeholder-Kommentare:
```bash
grep -rn "todo!\|unimplemented!\|STUB\|PLACEHOLDER\|FIXME\|HACK\|XXX" --include="*.rs" .
```

Für JEDEN gefundenen Stub: Entweder implementiere ihn oder entferne die Funktion wenn sie nicht gebraucht wird.

### 9.2 Scrollbars überall

Jeder Container der überlaufen kann, braucht:
```css
.fsn-scrollable {
    overflow-y: auto;
    scrollbar-width: thin;
    scrollbar-color: var(--fsn-border) transparent;
}
.fsn-scrollable::-webkit-scrollbar { width: 6px; }
.fsn-scrollable::-webkit-scrollbar-thumb {
    background: var(--fsn-border);
    border-radius: 3px;
}
.fsn-scrollable::-webkit-scrollbar-thumb:hover {
    background: var(--fsn-text-muted);
}
```

Füge `class: "fsn-scrollable"` zu jedem Container hinzu der scrollen können muss: Settings-Panels, Store-Listen, Sprach-Dropdown, Fenster-Inhalte.

### 9.3 Vollbild-Desktop

Das Root-Layout muss den gesamten Viewport füllen:
```css
html, body { margin: 0; padding: 0; width: 100%; height: 100%; overflow: hidden; }
.fsn-app { width: 100vw; height: 100vh; display: flex; flex-direction: column; }
.fsn-main { flex: 1; display: flex; overflow: hidden; }
.fsn-sidebar { width: 240px; flex-shrink: 0; overflow-y: auto; }
.fsn-content { flex: 1; overflow-y: auto; }
```

---

## Aufgabe 10: Extras

### 10.1 App-Gruppen mit Accordion

Auf der Home-Seite: Services in klappbare Gruppen zusammenfassen:

```rust
#[component]
fn AppGroup(title: String, children: Element, default_open: bool) -> Element {
    let is_open = use_signal(|| default_open);
    rsx! {
        div { class: "fsn-app-group",
            button {
                class: "fsn-app-group-header",
                onclick: move |_| is_open.toggle(),
                span { class: "fsn-app-group-arrow",
                    if *is_open.read() { "▼" } else { "▶" }
                }
                span { "{title}" }
            }
            if *is_open.read() {
                div { class: "fsn-app-group-content", {children} }
            }
        }
    }
}
```

### 10.2 Sidebar-Hover-Tooltips

Wenn die Sidebar collapsed ist, zeige Tooltips bei Hover:
```css
.fsn-sidebar.collapsed .fsn-sidebar-item:hover::after {
    content: attr(data-tooltip);
    position: absolute; left: 100%; top: 50%;
    transform: translateY(-50%);
    background: var(--fsn-bg-elevated);
    color: var(--fsn-text-primary);
    padding: 4px 10px; border-radius: 4px;
    white-space: nowrap; z-index: 100;
    box-shadow: var(--fsn-shadow);
}
```

### 10.3 Loading-States

Jeder async-Vorgang (Store laden, Service installieren) braucht einen Loading-State:
```rust
#[component]
fn LoadingSpinner(size: SpinnerSize) -> Element {
    let sz = match size {
        SpinnerSize::Sm => "16px",
        SpinnerSize::Md => "24px",
        SpinnerSize::Lg => "40px",
    };
    rsx! {
        div {
            class: "fsn-spinner",
            style: "width: {sz}; height: {sz};",
        }
    }
}
```

```css
.fsn-spinner {
    border: 2px solid var(--fsn-border);
    border-top-color: var(--fsn-primary);
    border-radius: 50%;
    animation: fsn-spin 0.6s linear infinite;
}
@keyframes fsn-spin { to { transform: rotate(360deg); } }
```

---

## Reihenfolge

```
1. Analyse (Schritt VORHER)
2. fsy → fsn Umbenennung (Aufgabe 1)
3. Dark + Light Theme (Aufgabe 2) — SOFORT sichtbare Verbesserung
4. Fenster-Buttons (Aufgabe 3)
5. Disabled-Buttons allgemein (Aufgabe 7)
6. Scrollbars + Vollbild (Aufgabe 9.2 + 9.3)
7. Store erweitern (Aufgabe 4)
8. Sprach-Dropdown (Aufgabe 5)
9. Widgets (Aufgabe 6)
10. Kanidm-Paket (Aufgabe 8)
11. Stubs aufräumen (Aufgabe 9.1)
12. Extras (Aufgabe 10)

Nach JEDEM Schritt: cargo check && cargo clippy
```
