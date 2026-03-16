# FreeSynergy Desktop — Arbeitsplan v3 (für Claude Code)

**Repo:** https://github.com/FreeSynergy/FreeSynergy.Desktop  
**Regeln:** English in code/comments. `cargo check` after every change. Replace ALL stubs. Remove dead code.

---

# OVERVIEW: Desktop Tabs / Sidebar

```
Sidebar:
  🏠 Home        ← Personal dashboard, widgets, quick-launch
  📋 Tasks       ← NEU: Automatisierungs-Pipelines (Serienbrief-System)
  🎛️ Conductor   ← Service + Bot Konfiguration (Variablen, Verbindungen)
  🤖 Bots        ← Bot Manager (Bots benutzen: Broadcast senden, Gruppen verwalten)
  📦 Store       ← Installieren (Plugins, Languages, Themes, Widgets, Bots)
  ⚙️ Settings    ← Themes, Shortcuts, Language, Profile
  ❓ Help        ← Docs, Shortcuts-Referenz, Tour

  Admin (wenn Admin-Rechte):
    🖥️ Hosts
    📊 Services
    🔐 Permissions
    📝 Logs
```

### Store → Conductor → Bot Manager: Drei klare Rollen

| Komponente | Rolle | Beispiel |
|---|---|---|
| **Store** | Laden: Finden + Installieren | "Kanidm installieren", "Broadcast Bot installieren" |
| **Conductor** | Konfigurieren: Variablen, Verbindungen, Start/Stop | "Kanidm: Domain setzen", "Broadcast Bot: Telegram-Token eingeben, Gruppen auswählen" |
| **Bot Manager** | Benutzen: Bots im Alltag bedienen | "Nachricht an alle Gruppen senden", "Gatekeeper: Wer wartet auf Freigabe?" |

---

# AUFGABE 1: Theme Manager + 5 Themes

(Identisch mit Plan v2 — Theme struct mit WindowChrome, ButtonStyle, etc.)
(5 Themes: Midnight Blue, Cloud White, Cupertino, Nordic, Rosé Pine)
(Live switching via CSS variables + data-theme attribute)

Themes im Store unter `shared/themes/`. Vorschau mit Farbpalette bei Auswahl.

---

# AUFGABE 2: Store — Alle Pakettypen

Store zeigt Tabs: **All | Plugins | Languages | Themes | Widgets | Bots | Bridges | Tasks**

Languages: Aus `Store/shared/i18n/` statt `Store/Node/i18n/`. Nur Englisch vorinstalliert, Rest über Store.

Language-Dropdown: Nur installierte Sprachen. "Install more..." Button am Ende. Scrollbar wenn > 8.

Widgets: Im Store unter `shared/widgets/`. Werden nach Installation auf Home platzierbar.

Tasks: Task-Templates im Store unter `shared/tasks/`. Vordefinierte Automatisierungen.

---

# AUFGABE 3: Store Directory → shared/

```
Store/
├── shared/              ← Für ALLE Projekte
│   ├── i18n/            ← Sprachpakete
│   ├── themes/          ← Themes
│   ├── widgets/         ← Widgets
│   ├── bots/            ← Bot-Definitionen
│   └── tasks/           ← Task-Templates
├── node/                ← Node-spezifisch
│   ├── modules/         ← Container-Plugins
│   └── bridges/
└── catalog.toml
```

---

# AUFGABE 4: Plugin-Definitionen

(Identisch mit Plan v2 — Kanidm, Outline, CryptPad, Zentinel)
(Jedes Plugin: manifest.toml, icon.svg, quadlet.container.tera, server.toml.tera, locales/)

---

# AUFGABE 5: Sidebar Auto-Hide + Bottom Bar entfernen

Sidebar verschwindet nach 2s, kommt bei Maus-am-Rand zurück. Animation 300ms.
Untere Taskleiste komplett entfernen.

---

# AUFGABE 6: Top Menu

```
FreeSynergy │ View │ Services │ Tools │ Help
```
(Identisch mit Plan v2 — vollständige Menüeinträge mit Shortcuts)

---

# AUFGABE 7: Profile als Ressource

## 7.1 Profil-Datenmodell

```rust
pub struct UserProfile {
    // From IAM (read-only when external)
    pub iam_id: Option<String>,
    pub username: String,
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub groups: Vec<String>,
    
    // User-editable (synced via CRDT)
    pub bio: Option<String>,
    pub location: Option<String>,
    pub timezone: String,
    pub language: String,
    pub website: Option<String>,
    pub social_links: HashMap<String, String>,
    pub linked_accounts: Vec<LinkedAccount>,
    
    // Personal resources — what this user "has"
    pub personal_capabilities: Vec<PersonalCapability>,
}

/// What a user personally brings into the system
pub enum PersonalCapability {
    /// User has a Telegram account → can receive personal bot messages
    MessengerAccount { platform: String, username: String, verified: bool },
    /// User has personal tasks in Vikunja
    TaskManager { service_id: String },
    /// User has personal mail
    Mailbox { service_id: String, address: String },
    /// User has a personal LLM assistant configured
    LlmAssistant { provider: String, model: String },
}
```

## 7.2 Profil als Ressource im System

The profile is a Resource like any Service. It can:
- **Offer data** to widgets: "My unread messages", "My tasks", "My calendar"
- **Connect to personal bots**: A UserBot that acts on behalf of this user
- **Connect to services**: Vikunja (personal tasks), Mail (personal inbox)

## 7.3 Account Linking (Token-Flow)

```
1. User clicks "Link Telegram" in Profile
2. Desktop generates 6-char token (expires 5 min)
3. Shows: "Send /verify TOKEN to @FreeSynergyBot on Telegram"
4. Bot receives /verify TOKEN + telegram_user_id
5. Bot calls Node API: validate(token, telegram_user_id)
6. Node links telegram_user_id → user profile
7. Desktop shows: "@username ✓ verified"
```

---

# AUFGABE 8: Bot Framework

## 8.1 Three-tier architecture

```
Store                    Conductor                Bot Manager
"Install Broadcast Bot"  "Set Telegram token,     "Type message,
                          select target groups"    hit Send"
```

## 8.2 Bot Definition (in Store)

```toml
[bot]
id = "broadcast"
name = "Broadcast Bot"
description = "Send messages to all connected groups"
icon = "megaphone"
type = "broadcast"           # broadcast | gatekeeper | monitor | digest | userbot

# What the Conductor needs to configure
[[bot.config_fields]]
name = "telegram_token"
label = "Telegram Bot Token"
type = "secret"
required_if = "telegram_enabled"

[[bot.config_fields]]
name = "telegram_groups"
label = "Telegram Groups"
type = "multi-select"
source = "telegram.groups"   # Populated from Telegram API

[[bot.config_fields]]
name = "matrix_rooms"
label = "Matrix Rooms"
type = "multi-select"
source = "matrix.rooms"

# What the Bot Manager UI shows to the user
[[bot.ui_fields]]
name = "message"
label = "Message"
type = "textarea"
placeholder = "Type your broadcast message..."

[[bot.ui_fields]]
name = "targets"
label = "Send to"
type = "multi-select-checkboxes"
source = "configured_targets"  # Shows all configured groups from Conductor
```

## 8.3 Conductor View for Bots

When you open a Bot in the Conductor, you see:
```
┌─ Conductor: Broadcast Bot ───────────────────────────┐
│                                                       │
│  Status: ● Running                                   │
│                                                       │
│  ── Channels ─────────────────────────────────────── │
│  ☑ Telegram                                          │
│    Bot Token: [••••••••••••••••••]                    │
│    Groups:                                            │
│      ☑ FreeSynergy Community (-100123...)             │
│      ☑ Helfa Köln (-100456...)                       │
│      ☐ Test Group (-100789...)                       │
│                                                       │
│  ☑ Matrix                                            │
│    Homeserver: [matrix.example.com]                   │
│    Bot Token: [••••••••••••••••••]                    │
│    Rooms:                                             │
│      ☑ #general:example.com                          │
│      ☑ #announcements:example.com                    │
│                                                       │
│  ☐ Discord (not configured)                          │
│                                                       │
│  [Save] [Test Connection]                            │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## 8.4 Bot Manager View

When you open a Bot in the Bot Manager, you see the USAGE interface:

```
┌─ Bots: Broadcast ────────────────────────────────────┐
│                                                       │
│  ┌─────────────────────────────────────────────────┐ │
│  │ Type your message here...                        │ │
│  │                                                  │ │
│  │                                                  │ │
│  └─────────────────────────────────────────────────┘ │
│                                                       │
│  Send to:                                            │
│  ☑ Telegram: FreeSynergy Community                   │
│  ☑ Telegram: Helfa Köln                              │
│  ☑ Matrix: #general                                  │
│  ☑ Matrix: #announcements                            │
│                                                       │
│  [📤 Send Broadcast]                                 │
│                                                       │
│  ── Recent Broadcasts ────────────────────────────── │
│  📤 "Neues Update verfügbar..." — 2h ago — 4 groups │
│  📤 "Treffen am Samstag..." — 1d ago — 2 groups     │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## 8.5 Gatekeeper Bot

Conductor config:
```
┌─ Conductor: Gatekeeper Bot ──────────────────────────┐
│                                                       │
│  ── Telegram Groups ──────────────────────────────── │
│  Group: FreeSynergy Community                        │
│  Access Rule: [Kanidm Group ▼] → "members"           │
│  → Only users in Kanidm group "members" can join     │
│                                                       │
│  Group: Helfa Köln                                   │
│  Access Rule: [Kanidm Group ▼] → "helfa-koeln"      │
│                                                       │
│  Timeout: [5] minutes (kick unverified after)        │
│  Verify URL: https://node.example.com/verify         │
│                                                       │
└───────────────────────────────────────────────────────┘
```

Bot Manager view for Gatekeeper:
```
┌─ Bots: Gatekeeper ───────────────────────────────────┐
│                                                       │
│  ── Pending Verification ─────────────────────────── │
│  🔒 @alice_t (Telegram) — waiting 2m — [✓ Allow] [✗] │
│  🔒 @bob_99 (Telegram) — waiting 4m — [✓ Allow] [✗]  │
│                                                       │
│  ── Recent Activity ──────────────────────────────── │
│  ✓ @charlie joined "Helfa Köln" — 1h ago             │
│  ✗ @spammer123 kicked (timeout) — 3h ago             │
│  ✓ @dave joined "Community" — 5h ago                 │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## 8.6 Personal UserBot (LLM-assisted)

A special bot type that acts on behalf of a single user:

```toml
[bot]
id = "personal-assistant"
name = "Personal Assistant"
type = "userbot"
description = "LLM-powered assistant that can answer on your behalf"

[[bot.config_fields]]
name = "llm_provider"
label = "LLM Provider"
type = "select"
options = ["ollama", "claude", "openai-compatible"]

[[bot.config_fields]]
name = "auto_reply_channels"
label = "Auto-reply in these channels"
type = "multi-select"
source = "user.messenger_accounts"

[[bot.config_fields]]
name = "system_prompt"
label = "Instructions for the assistant"
type = "textarea"
default = "You are a helpful assistant for {user.display_name}. Answer questions about their projects and schedule."
```

---

# AUFGABE 9: Configurable Keyboard Shortcuts

## 9.1 Action Registry

Every action in the Desktop gets a unique ID:

```rust
pub struct ActionDef {
    pub id: String,              // "view.sidebar.toggle"
    pub label_key: String,       // i18n key: "action-toggle-sidebar"
    pub category: String,        // "View", "Services", "Navigation"
    pub default_shortcut: Option<String>, // "Ctrl+B"
    pub current_shortcut: Option<String>, // User-customized
}

// Register ALL actions at startup
pub fn register_actions() -> Vec<ActionDef> {
    vec![
        ActionDef { id: "app.settings".into(), label_key: "action-open-settings".into(), category: "App".into(), default_shortcut: Some("Ctrl+,".into()), .. },
        ActionDef { id: "app.quit".into(), label_key: "action-quit".into(), category: "App".into(), default_shortcut: Some("Ctrl+Q".into()), .. },
        ActionDef { id: "view.sidebar.toggle".into(), label_key: "action-toggle-sidebar".into(), category: "View".into(), default_shortcut: Some("Ctrl+B".into()), .. },
        ActionDef { id: "view.fullscreen".into(), label_key: "action-fullscreen".into(), category: "View".into(), default_shortcut: Some("F11".into()), .. },
        ActionDef { id: "store.open".into(), label_key: "action-open-store".into(), category: "Tools".into(), default_shortcut: Some("Ctrl+S".into()), .. },
        ActionDef { id: "store.install".into(), label_key: "action-install".into(), category: "Tools".into(), default_shortcut: Some("Ctrl+I".into()), .. },
        ActionDef { id: "help.open".into(), label_key: "action-open-help".into(), category: "Help".into(), default_shortcut: Some("F1".into()), .. },
        ActionDef { id: "window.close".into(), label_key: "action-close-window".into(), category: "Window".into(), default_shortcut: Some("Escape".into()), .. },
        ActionDef { id: "tasks.open".into(), label_key: "action-open-tasks".into(), category: "Tools".into(), default_shortcut: Some("Ctrl+T".into()), .. },
        ActionDef { id: "bots.open".into(), label_key: "action-open-bots".into(), category: "Tools".into(), default_shortcut: None, .. },
        ActionDef { id: "conductor.open".into(), label_key: "action-open-conductor".into(), category: "Tools".into(), default_shortcut: None, .. },
        // ... every view, every action, every window gets an ID
    ]
}
```

## 9.2 Settings: Shortcut Editor

In Settings, show a list of ALL actions grouped by category. Each row shows the action name and a clickable shortcut field. Clicking it puts it in "recording" mode — user presses the desired key combo — it saves.

```
┌─ Settings: Keyboard Shortcuts ───────────────────────┐
│                                                       │
│  🔍 [Search actions...                          ]    │
│                                                       │
│  ── App ──────────────────────────────────────────── │
│  Open Settings          [Ctrl+,    ] [Reset]         │
│  Quit                   [Ctrl+Q    ] [Reset]         │
│                                                       │
│  ── View ─────────────────────────────────────────── │
│  Toggle Sidebar         [Ctrl+B    ] [Reset]         │
│  Fullscreen             [F11       ] [Reset]         │
│                                                       │
│  ── Tools ────────────────────────────────────────── │
│  Open Store             [Ctrl+S    ] [Reset]         │
│  Open Tasks             [Ctrl+T    ] [Reset]         │
│  Install Package        [Ctrl+I    ] [Reset]         │
│  Open Bot Manager       [  None    ] [Set]           │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## 9.3 Help: Shortcut Reference

In Help, show the SAME list but read-only. This is auto-generated from the Action Registry — never manually maintained.

---

# AUFGABE 10: Widget System (Personal Layer)

## 10.1 Widgets = Personal

Widgets are the PERSONAL layer of the Desktop. They show data relevant to ME, not to the organization:
- My messages (from Mail, Telegram, Matrix — via fsn-bus filtered to my accounts)
- My tasks (from Vikunja, filtered to my user)
- Clock / Date
- System info (if I'm an admin)
- Weather (personal location)
- Quick notes

Services/Bots/Tasks = ORGANIZATIONAL layer (shared).

## 10.2 Widget Placement via Edit Mode

Right-click on desktop background → "Edit Desktop" enters edit mode:

```
┌─ Desktop (Edit Mode) ─────────────────── [✓ Done] ──┐
│                                                       │
│  ┌──────────┐  ┌──────────┐  ╔══════════════════╗   │
│  │ 🕐 Clock │  │ 📊 Sys   │  ║  + Add Widget    ║   │
│  │  14:32   │  │ CPU: 23% │  ║                   ║   │
│  │  Monday  │  │ RAM: 4GB │  ║  [Clock        ]  ║   │
│  └──┤resize├┘  └──┤resize├┘  ║  [System Info  ]  ║   │
│     ↕drag↕        ↕drag↕     ║  [Messages     ]  ║   │
│                               ║  [My Tasks     ]  ║   │
│  ┌──────────────────────┐    ║  [Quick Notes  ]  ║   │
│  │ 📬 Messages          │    ║  [Weather      ]  ║   │
│  │ ✉ 3 new emails       │    ║  ↓ scroll      ║   │
│  │ 💬 2 Telegram msgs    │    ╚══════════════════╝   │
│  └──────────────────────┘                             │
│                                                       │
│  🗑️ Drag here to remove                              │
│                                                       │
└───────────────────────────────────────────────────────┘
```

- Right-click background → "Edit Desktop"
- "Add Widget" panel appears (scrollable list)
- Click a widget → it appears on desktop, can be dragged
- Resize handles on corners
- Drag to trash to remove
- "Done" button saves layout and exits edit mode

## 10.3 Messages Widget

The Messages widget aggregates personal messages from ALL connected channels:

```rust
#[component]
fn MessagesWidget() -> Element {
    // Subscribe to bus events filtered to current user's accounts
    let messages = use_bus_subscription(BusFilter {
        event_types: vec![EventType::MessageReceived],
        target_user: Some(current_user_id()),
    });
    
    rsx! {
        div { class: "fsn-widget fsn-widget-messages",
            h3 { "Messages" }
            for msg in messages.read().iter().take(10) {
                div { class: "fsn-message-row",
                    span { class: "fsn-message-platform", "{msg.platform_icon()}" }
                    span { class: "fsn-message-sender", "{msg.sender}" }
                    span { class: "fsn-message-preview", "{msg.preview(50)}" }
                    span { class: "fsn-message-time", "{msg.time_ago()}" }
                }
            }
        }
    }
}
```

## 10.4 My Tasks Widget

```rust
#[component]
fn MyTasksWidget() -> Element {
    // Fetches personal tasks from connected task service (Vikunja, etc.)
    let tasks = use_service_data("tasks.mine", current_user_id());
    
    rsx! {
        div { class: "fsn-widget fsn-widget-tasks",
            h3 { "My Tasks" }
            for task in tasks.read().iter().take(8) {
                div { class: "fsn-task-row",
                    input { r#type: "checkbox", checked: task.done }
                    span { class: if task.overdue { "fsn-text-error" } else { "" },
                        "{task.title}"
                    }
                    if let Some(due) = &task.due_date {
                        span { class: "fsn-task-due", "{due.format_relative()}" }
                    }
                }
            }
        }
    }
}
```

---

# AUFGABE 11: Tasks — Automatisierungs-Pipelines

## 11.1 Concept: Data Offers + Data Accepts + Field Mapping

Every resource declares its **Data Offers** (what it can provide, with concrete fields) and **Data Accepts** (what it can receive):

```rust
/// A concrete data offer from a service — NOT just "JSON" but actual fields
pub struct DataOffer {
    pub id: String,             // "repos.list"
    pub service: String,        // "forgejo"
    pub label_key: String,      // i18n: "offer-repos-list"
    pub description_key: String,
    pub returns: DataShape,     // The concrete shape of the data
    pub trigger: DataTrigger,   // When is this data available?
}

pub enum DataTrigger {
    OnDemand,                   // Pull: request it when needed
    OnEvent(String),            // Push: triggered by a bus event (e.g. "commit-pushed")
    Scheduled(String),          // Cron: "0 8 * * *" (daily at 8am)
}

/// Concrete shape — NOT "JSON" but actual typed fields
pub struct DataShape {
    pub is_list: bool,          // true = returns multiple items
    pub fields: Vec<DataField>,
}

pub struct DataField {
    pub name: String,           // "name"
    pub label_key: String,      // i18n: "field-repo-name"
    pub field_type: FieldType,  // String, Integer, Url, Markdown, DateTime, ...
    pub example: String,        // "my-cool-project"
}
```

## 11.2 Concrete Data Offers per Service

### Kanidm offers:
```
users.list → [
    { username: String, display_name: String, email: Email, groups: String[], created: DateTime }
]

groups.list → [
    { name: String, members: String[], description: String }
]

login.recent → [
    { username: String, timestamp: DateTime, success: Boolean, ip: String }
]
```

### Forgejo offers:
```
repos.list → [
    { name: String, description: String, url: Url, default_branch: String, stars: Integer, language: String, created: DateTime, updated: DateTime }
]

repos.readme → [
    { repo_name: String, content: Markdown }
]

commits.recent → [
    { repo: String, hash: String, message: String, author: String, date: DateTime }
]

issues.open → [
    { repo: String, number: Integer, title: String, body: Markdown, author: String, labels: String[], created: DateTime }
]
```

### Vikunja offers:
```
tasks.mine → [
    { title: String, description: Markdown, done: Boolean, due_date: DateTime?, priority: Integer, project: String, labels: String[] }
]

tasks.overdue → [
    { title: String, due_date: DateTime, project: String, assignee: String }
]

projects.list → [
    { name: String, description: String, task_count: Integer, done_count: Integer }
]
```

### Outline accepts:
```
document.create ← {
    title: String (required),
    body: Markdown (required),
    collection: String (required — which wiki section),
    publish: Boolean (default: true)
}

document.update ← {
    id: String (required),
    title: String?,
    body: Markdown?,
    append: Markdown? (add to end instead of replace)
}
```

### uMap accepts:
```
point.create ← {
    lat: Float (required),
    lon: Float (required),
    name: String (required),
    description: String?,
    popup_content: Markdown?,
    icon_url: Url?
}
```

## 11.3 Task Builder (Visual Pipeline Editor)

The Tasks tab shows a visual editor where users connect data offers to data accepts:

```
┌─ Tasks ──────────────────────────────────────────────┐
│                                                       │
│  [+ New Task]  [📋 My Tasks]  [📦 Task Templates]    │
│                                                       │
│  ── Task: "Service Docs ins Wiki" ────────────────── │
│                                                       │
│   SOURCE                    TARGET                    │
│  ┌─────────────┐           ┌─────────────┐          │
│  │ Forgejo      │           │ Outline      │          │
│  │ repos.list   │──────────▶│ doc.create   │          │
│  └─────────────┘           └─────────────┘          │
│                                                       │
│  ── Field Mapping ────────────────────────────────── │
│                                                       │
│  Source Field       →  Target Field                  │
│  ─────────────────────────────────────               │
│  repo.name          →  title  (as "Repo: {value}")   │
│  repo.readme        →  body                          │
│  (fixed)            →  collection ("Documentation")  │
│  (fixed)            →  publish (true)                │
│                                                       │
│  Trigger: [● On Event: repo-created  ▼]             │
│           [○ Scheduled: __________ ]                 │
│           [○ Manual only          ]                  │
│                                                       │
│  [▶ Run Now]  [💾 Save]  [🗑 Delete]                │
│                                                       │
└───────────────────────────────────────────────────────┘
```

## 11.4 Field Mapping with Templates

Each field mapping can use a Tera template for transformation:

```rust
pub struct FieldMapping {
    pub source_field: Option<String>,  // None = fixed value
    pub target_field: String,
    pub transform: FieldTransform,
}

pub enum FieldTransform {
    /// Direct copy: source → target
    Direct,
    /// Template: "Repository: {{ value }}" 
    Template(String),
    /// Fixed value: always "Documentation"
    Fixed(String),
    /// Combine multiple fields: "{{ name }} by {{ author }}"
    Combine(String),
}
```

## 11.5 Pre-built Task Templates (from Store)

```
shared/tasks/
├── service-docs-to-wiki/     ← "Write service documentation to wiki"
│   └── manifest.toml
├── new-user-welcome/          ← "Send welcome message when new user created"  
│   └── manifest.toml
├── daily-task-digest/         ← "Send daily overdue tasks summary via chat"
│   └── manifest.toml
├── commit-changelog/          ← "Write commits to wiki changelog page"
│   └── manifest.toml
├── group-to-map-wiki-git/     ← "Create group → map point + wiki page + git repo"
│   └── manifest.toml
└── login-alert/               ← "Alert on failed logins"
    └── manifest.toml
```

Each template is a pre-configured Task with the field mappings already set. User just selects source/target services and it works.

## 11.6 LLM-assisted Task Creation

If the user doesn't want to build a task manually:

```
User: "I want every new Forgejo commit to be logged in the wiki"
LLM: "I'll create a task for you:
      Source: Forgejo → commits.recent (trigger: on commit-pushed event)
      Target: Outline → document.update (append to 'Changelog' page)
      Mapping: commit.message → body (as '- {hash}: {message} by {author}')
      
      Shall I create this task?"
User: "Yes"
→ Task is created and activated
```

---

# AUFGABE 12: Notifications Bell

Top bar bell icon showing recent events:

```rust
#[component]
fn NotificationBell() -> Element {
    let notifications = use_signal(|| Vec::<Notification>::new());
    let unread_count = notifications.read().iter().filter(|n| !n.read).count();
    let show_panel = use_signal(|| false);
    
    rsx! {
        div { class: "fsn-notification-bell",
            button {
                onclick: move |_| show_panel.toggle(),
                "🔔"
                if unread_count > 0 {
                    span { class: "fsn-badge fsn-badge-error", "{unread_count}" }
                }
            }
            if *show_panel.read() {
                div { class: "fsn-notification-panel",
                    h3 { "Notifications" }
                    div { class: "fsn-scrollable",
                        style: "max-height: 400px;",
                        for notif in notifications.read().iter() {
                            NotificationRow { notification: notif.clone() }
                        }
                    }
                }
            }
        }
    }
}
```

---

# AUFGABE 13: Context Menu + Edit Mode + Extras

## 13.1 Right-click context menus

Service cards: Start, Stop, Restart, Configure, Logs, Remove
Store items: Install, View Details, Visit Homepage  
Desktop background: Edit Desktop, Add Widget, Settings

## 13.2 Desktop background selector

In Edit Mode, allow changing the desktop background:
- Solid color (from theme)
- Gradient
- Image (upload or URL)

Saved in settings.

## 13.3 Loading states everywhere

Every async operation shows a spinner. Use the `LoadingSpinner` component consistently.

---

# EXECUTION ORDER

```
 1. Theme Manager + 5 themes (Aufgabe 1)
 2. Sidebar auto-hide + remove bottom bar (Aufgabe 5)
 3. Top menu with entries (Aufgabe 6)
 4. Configurable Shortcuts system (Aufgabe 9)
 5. Store: all package types + shared/ directory (Aufgabe 2 + 3)
 6. Language dropdown: only installed (Aufgabe 2)
 7. Plugin definitions: Kanidm, Outline, CryptPad, Zentinel (Aufgabe 4)
 8. Widget system with Edit Mode (Aufgabe 10)
 9. Profile as resource + account linking (Aufgabe 7)
10. Bot framework: Conductor + Bot Manager views (Aufgabe 8)
11. Broadcast Bot + Gatekeeper Bot (Aufgabe 8.3-8.5)
12. Tasks tab: Data Offers/Accepts + Field Mapping (Aufgabe 11)
13. Task Templates from Store (Aufgabe 11.5)
14. Notifications bell (Aufgabe 12)
15. Context menus + Edit mode extras (Aufgabe 13)
16. Personal UserBot with LLM (Aufgabe 8.6)
17. LLM-assisted Task creation (Aufgabe 11.6)
18. Remove ALL stubs/dead code

After EACH step: cargo check && cargo clippy
Commit with clear message.
```
