# FreeSynergy — Installationsprozess v2 + Claude-Code-Arbeitspakete

**Ergänzung zu:** FREESYNERGY-PLAN-v5-FINAL.md  
**Version:** 2.0 — Alle 12 Feedback-Punkte eingearbeitet

---

## Änderungen gegenüber v1

| # | Punkt | Änderung |
|---|---|---|
| 1 | Node + Desktop zusammen | Node-Installation geht auch via Desktop (TUI/GUI/WGUI) |
| 2 | IAM-Auswahl | Beliebiger IAM installierbar, Kanidm ist Empfehlung |
| 3 | Typ-System + Capability-Matching | Pakete zeigen was sie können/brauchen, automatische Service-Verknüpfung |
| 4 | Variablen-Typ-System | Rollen-Typen für Felder (SMTP.host, IAM.oauth_url, Secret, ...) |
| 5 | Worker/Scaling | Services auf mehrere Hosts verteilen, Worker-Modus |
| 6 | Podman OHNE Socket | Kein Socket. Quadlet + systemd + CLI only |
| 7 | Store-Architektur | Ein Store für alles, Namespaces für Projekte |
| 8 | Store-Berechtigungen | Rollenbasiert: Admin, Node-Admin, User |
| 9 | Dynamische Capabilities | Mehr Services = mehr Möglichkeiten |
| 10 | Typ-Berechtigungen | Später: einheitliche Permission-Oberfläche |
| 11 | Vergessenes | Recovery-Key, Auto-Discovery, Migration bestehender Services |
| 12 | Claude-Code-Plan | Arbeitspakete für die Umsetzung |

---

# TEIL 1 — INSTALLATION

## 1.1 Node-Installation auch via Desktop

Der Node wird NICHT nur per CLI installiert. Es gibt drei Wege:

```
fsn install              ← CLI (Terminal, SSH, Headless)
fsn install --tui        ← TUI (Terminal mit Oberfläche)
fsn install --gui        ← Desktop-App (Dioxus WGUI)
```

Alle drei durchlaufen denselben Wizard mit denselben Schritten — nur die Darstellung ist anders. Der Wizard-Code liegt in `fsn-wizard` und ist UI-unabhängig. Die UI-Schicht (CLI/TUI/GUI) ruft nur die Wizard-Logik auf.

## 1.2 IAM — Frei wählbar

```
┌─ IAM auswählen ─────────────────────────────────────┐
│                                                       │
│  Wie sollen sich Benutzer anmelden?                   │
│                                                       │
│  ○ Aus dem Store installieren:                        │
│    ⭐ Kanidm (empfohlen)     OIDC ✓ SCIM ✓ MFA ✓   │
│       KeyCloak                OIDC ✓ SCIM ✓ Multi ✓  │
│       Authentik               OIDC ✓ SCIM ✓ Flows ✓  │
│       LLDAP                   LDAP ✓ SCIM ✗ Leicht ✓ │
│                                                       │
│  ○ Extern (schon vorhanden)                          │
│                                                       │
│  ○ Kein IAM (nur lokal)                              │
│                                                       │
└───────────────────────────────────────────────────────┘
```

Jedes IAM-Paket zeigt seine **Capabilities** an — damit man sieht was es kann und was nicht.

---

# TEIL 2 — TYP-SYSTEM & CAPABILITY-MATCHING

## 2.1 Das Grundprinzip

Jeder Service meldet sich mit seinen **Capabilities** (was er kann) und **Requirements** (was er braucht) an. Das System matcht automatisch.

```
Service "Kanidm" meldet an:
  ICH BIETE: oidc-provider, scim-server, mfa, radius, multi-tenant
  ICH BRAUCHE: database.postgres

Service "KeyCloak" meldet an:
  ICH BIETE: oidc-provider, saml, mfa, multi-tenant
  ICH BRAUCHE: database.postgres
  ICH BIETE NICHT: scim-server    ← Wichtig! Wird angezeigt!

Service "BookStack" meldet an:
  ICH BRAUCHE: iam.oauth-url, iam.client-id, iam.client-secret
  ICH BRAUCHE: smtp.host, smtp.port, smtp.username, smtp.password
  ICH BRAUCHE: database.mysql
```

Wenn BookStack installiert wird und Kanidm schon läuft, weiß das System automatisch: "Kanidm bietet `iam.*` an → BookStack bekommt die Werte automatisch."

## 2.2 Capability-Katalog (im Store)

```toml
# Store/types/iam.toml — Was ein IAM-Service können KANN

[type.iam]
name_key = "type-iam-name"
description_key = "type-iam-description"

# Mögliche Capabilities (nicht jeder IAM hat alle)
[type.iam.capabilities]
oidc-provider = { description_key = "cap-oidc-provider", required = true }
scim-server = { description_key = "cap-scim-server", required = false }
saml = { description_key = "cap-saml", required = false }
mfa = { description_key = "cap-mfa", required = false }
multi-tenant = { description_key = "cap-multi-tenant", required = false }
radius = { description_key = "cap-radius", required = false }
ldap = { description_key = "cap-ldap", required = false }
webauthn = { description_key = "cap-webauthn", required = false }

# Was ein IAM-Service über den Bus bereitstellt
[type.iam.bus]
emits = ["user-created", "user-updated", "user-deleted", "login-success", "login-failed"]
listens = ["user-provisioned", "group-changed"]

# APIs die ein IAM-Service hat
[type.iam.api]
oidc-discovery = { path = "/.well-known/openid-configuration", method = "GET" }
oauth-token = { path = "/oauth2/token", method = "POST" }
scim-users = { path = "/scim/v2/Users", methods = ["GET", "POST", "PUT", "DELETE"] }
```

## 2.3 Paket-Manifest mit Capability-Deklaration

```toml
# Kanidm zeigt: das kann ich, das kann ich nicht
[package.capabilities]
oidc-provider = true
scim-server = true
mfa = true
multi-tenant = true    # ← KeyCloak hat das auch
radius = true
ldap = false           # ← Kanidm hat kein LDAP
saml = false           # ← Kanidm hat kein SAML
webauthn = true

# Im Store/Desktop sieht man dann:
# Kanidm:    OIDC ✓  SCIM ✓  MFA ✓  Multi ✓  RADIUS ✓  LDAP ✗  SAML ✗  WebAuthn ✓
# KeyCloak:  OIDC ✓  SCIM ✗  MFA ✓  Multi ✓  RADIUS ✗  LDAP ✓  SAML ✓  WebAuthn ✓
```

## 2.4 Automatische Service-Verknüpfung (das Killer-Feature)

**Beispiel: "Gruppe in Köln anlegen"**

Der Benutzer füllt EIN Formular aus:

```
┌─ Neue Gruppe anlegen ────────────────────────────────┐
│                                                       │
│  Name:        [Helfa Köln__________________]          │
│  Beschreibung:[Gemeinschaft in Köln_______ ]          │
│  Standort:    [📍 Köln, Domplatz___________ ]         │
│                                                       │
│  Automatische Aktionen:                               │
│  ☑ Punkt auf der Karte (uMap)                        │
│  ☑ Wiki-Artikel erstellen (Outline)                  │
│  ☑ Git-Repository erstellen (Forgejo)                │
│  ☑ Chat-Raum erstellen (Matrix)                      │
│  ☐ Aufgaben-Board erstellen (Vikunja)                │
│                                                       │
│  [Anlegen]  [Abbrechen]                              │
│                                                       │
└───────────────────────────────────────────────────────┘
```

Was passiert:

```
1. Formular wird abgeschickt
2. fsn-bus Event: GroupCreated { name, description, location, ... }
3. Routing-Regeln matchen:

   → uMap (hat Capability: map.create-point)
     API-Call: POST /api/map/point { lat, lon, name, description, wiki_url }
   
   → Outline (hat Capability: wiki.create-page)
     API-Call: POST /api/documents { title, body_template, links: [map_url, git_url] }
   
   → Forgejo (hat Capability: git.create-repo)
     API-Call: POST /api/v1/orgs/helfa/repos { name, description }
   
   → Matrix (hat Capability: chat.create-room)
     API-Call: POST /_matrix/client/v3/createRoom { name, topic }

4. Jeder Service gibt seine URL zurück
5. Alle URLs werden kreuzverlinkt:
   - Karte zeigt Link zum Wiki und Git
   - Wiki zeigt Link zur Karte und Git
   - Git zeigt Link zum Wiki und Karte
   - Chat-Raum hat Links zu allem
```

**Das funktioniert, weil jeder Service seine Capabilities deklariert und der Bus die Verknüpfung macht.**

---

# TEIL 3 — VARIABLEN-TYP-SYSTEM

## 3.1 Jede Variable hat einen Typ UND eine Rolle

```toml
[[variables]]
name = "OAUTH_ISSUER_URL"
label_key = "var-oauth-issuer-url"
type = "url"                          # Basis-Typ: string, integer, boolean, url, email, ip, port, path, secret, password
role = "iam.oidc-discovery-url"       # Rollen-Typ: welche Capability liefert diesen Wert?
priority = 1
required = true

[[variables]]
name = "SMTP_HOST"
label_key = "var-smtp-host"
type = "hostname"
role = "smtp.host"                    # Braucht einen SMTP-Service
priority = 1
required = true

[[variables]]
name = "SMTP_PASSWORD"
label_key = "var-smtp-password"
type = "secret"                       # Typ "secret" → wird verschlüsselt gespeichert (age)
role = "smtp.password"                # Kein Vault nötig — der Typ "secret" IST die Verschlüsselung
priority = 1
required = true

[[variables]]
name = "DATABASE_URL"
label_key = "var-database-url"
type = "connection-string"
role = "database.postgres.url"        # Braucht einen Postgres-Service
priority = 1
required = true

[[variables]]
name = "MAX_UPLOAD_SIZE"
label_key = "var-max-upload-size"
type = "bytes"                        # Basis-Typ mit Einheit (MB, GB, ...)
role = ""                             # Keine Rolle — reine Konfiguration
priority = 3
default = "100MB"
```

## 3.2 Basis-Typen

| Typ | Beschreibung | Validierung | Verschlüsselung |
|---|---|---|---|
| `string` | Freitext | Länge | Nein |
| `integer` | Ganzzahl | Min/Max | Nein |
| `boolean` | Ja/Nein | — | Nein |
| `url` | URL | URL-Format | Nein |
| `email` | E-Mail | E-Mail-Format | Nein |
| `hostname` | Hostname/Domain | DNS-Format | Nein |
| `ip` | IP-Adresse | IPv4/IPv6 | Nein |
| `port` | Port-Nummer | 1–65535 | Nein |
| `path` | Dateipfad | Existenz prüfbar | Nein |
| `connection-string` | DB-Verbindung | Format | Nein |
| `bytes` | Größe (MB, GB) | Einheit | Nein |
| `duration` | Zeitdauer (5m, 1h) | Einheit | Nein |
| `cron` | Cron-Ausdruck | Cron-Format | Nein |
| `select` | Auswahl aus Liste | Options-Liste | Nein |
| **`secret`** | **Geheimnis** | — | **Ja (age)** |
| **`password`** | **Passwort** | Stärke-Check | **Ja (age)** |
| **`api-key`** | **API-Schlüssel** | — | **Ja (age)** |
| **`certificate`** | **Zertifikat** | PEM-Format | **Ja (age)** |
| **`private-key`** | **Privater Schlüssel** | — | **Ja (age)** |

**Alles was `secret`, `password`, `api-key`, `certificate` oder `private-key` ist, wird automatisch verschlüsselt gespeichert.** Kein separates Vault nötig — der Typ IST die Sicherheit.

## 3.3 Rollen-Typen

```
Rolle = "service-typ.capability-wert"

Beispiele:
  iam.oidc-discovery-url     → Die OIDC-Discovery-URL des IAM-Services
  iam.client-id              → Die Client-ID beim IAM
  iam.client-secret          → Das Client-Secret beim IAM (Typ: secret)
  smtp.host                  → Der SMTP-Hostname
  smtp.port                  → Der SMTP-Port
  smtp.username              → SMTP-Benutzername
  smtp.password              → SMTP-Passwort (Typ: secret)
  database.postgres.url      → PostgreSQL Connection-String
  database.postgres.host     → PostgreSQL Hostname
  database.mysql.url         → MySQL Connection-String
  git.api-url                → Git-API-URL
  wiki.api-url               → Wiki-API-URL
  map.api-url                → Map-API-URL
  chat.homeserver-url        → Matrix-Homeserver-URL
```

## 3.4 Automatisches Ausfüllen

Wenn ein Service installiert wird und eine Variable mit Rolle hat, zeigt die UI:

```
┌─ BookStack konfigurieren ────────────────────────────┐
│                                                       │
│  OAUTH_ISSUER_URL:  [https://auth.example.com____]   │
│    Rolle: iam.oidc-discovery-url                      │
│    Quelle: [Kanidm (lokal)          ▼]               │
│            [KeyCloak (extern)        ]                │
│    → Automatisch befüllt!                             │
│                                                       │
│  SMTP_HOST:         [mail.example.com____________]   │
│    Rolle: smtp.host                                   │
│    Quelle: [Stalwart (lokal)         ▼]              │
│    → Automatisch befüllt!                             │
│                                                       │
│  DATABASE_URL:      [postgres://bookstack:***@...]   │
│    Rolle: database.postgres.url                       │
│    Quelle: [PostgreSQL (lokal)       ▼]              │
│    → Automatisch befüllt!                             │
│                                                       │
└───────────────────────────────────────────────────────┘
```

Wenn es mehrere Services gibt die dieselbe Rolle anbieten (z.B. zwei IAM-Services), kann der Benutzer wählen.

---

# TEIL 4 — WORKER/SCALING

## 4.1 Service auf mehrere Hosts verteilen

```toml
# Im Package-Manifest
[package.scaling]
supports_workers = true           # Kann als Worker laufen?
supports_horizontal_scaling = true # Kann mehrere Instanzen parallel?
min_instances = 1
max_instances = 0                 # 0 = unbegrenzt
worker_mode = "stateless"         # stateless | stateful | leader-follower
```

Wenn `supports_workers = false` → das Paket zeigt das klar an und die Option wird im UI nicht angeboten.

## 4.2 Wenn ein Service mehrfach vorkommt

```
┌─ Forgejo wird auf einem zweiten Host erkannt ────────┐
│                                                       │
│  Forgejo läuft bereits auf node1.example.com.         │
│  Wie soll die neue Instanz auf node2 funktionieren?  │
│                                                       │
│  ○ Worker (Lastverteilung mit node1)                 │
│    → Beide teilen sich die Arbeit                    │
│                                                       │
│  ○ Eigenständig (separater Service)                  │
│    → Unabhängig von der Instanz auf node1            │
│                                                       │
│  ○ Mirror (Read-Only Replik)                         │
│    → Nur Lesezugriff, Backup-Zweck                   │
│                                                       │
│  ℹ Forgejo unterstützt: Worker ✓ Eigenständig ✓      │
│    Mirror ✗                                           │
│                                                       │
└───────────────────────────────────────────────────────┘
```

---

# TEIL 5 — PODMAN OHNE SOCKET

Podman läuft **IMMER ohne Socket**. Kein `podman.sock`, kein API-Zugriff von außen.

Container-Management geht ausschließlich über:
- **Quadlet** (`.container`-Dateien → systemd-Generator)
- **systemctl** (start, stop, restart, status)
- **Podman CLI** (nur für Administration, nie von Services aufgerufen)

```rust
// fsn-container: Kein bollard, kein Socket!
// Stattdessen: Quadlet-Dateien generieren + systemctl aufrufen

pub struct QuadletManager {
    quadlet_dir: PathBuf,  // ~/.config/containers/systemd/
}

impl QuadletManager {
    /// Quadlet-Datei erstellen
    pub fn create_quadlet(&self, service: &ServiceConfig) -> Result<PathBuf>;
    
    /// systemctl --user daemon-reload
    pub fn reload_daemon(&self) -> Result<()>;
    
    /// systemctl --user start <service>.service
    pub fn start_service(&self, name: &str) -> Result<()>;
    
    /// systemctl --user stop <service>.service
    pub fn stop_service(&self, name: &str) -> Result<()>;
    
    /// systemctl --user status <service>.service
    pub fn service_status(&self, name: &str) -> Result<ServiceStatus>;
    
    /// journalctl --user -u <service>.service
    pub fn service_logs(&self, name: &str, lines: usize) -> Result<Vec<String>>;
}
```

---

# TEIL 6 — STORE-ARCHITEKTUR

## 6.1 Ein Store, Namespaces für Projekte

Statt `Node.Store`, `Wiki.Store`, `Decidim.Store` → **EIN Store mit Namespaces**:

```
FreeSynergy/Store
├── shared/                    ← Für ALLE Projekte (Sprachen, Themes, CSS, Logos)
│   ├── i18n/                  ← Sprach-Snippets (von fsn-i18n)
│   ├── themes/                ← Themes
│   └── assets/                ← Logos, Icons, CSS
│
├── node/                      ← Nur für FreeSynergy.Node
│   ├── modules/               ← Container-Module (Kanidm, Forgejo, ...)
│   ├── bridges/               ← Bridge-Plugins
│   └── bot-commands/          ← Bot-Commands
│
├── desktop/                   ← Nur für FreeSynergy.Desktop
│   ├── widgets/               ← Desktop-Widgets
│   └── views/                 ← Custom Views
│
├── wiki/                      ← Nur für Wiki.rs
│   ├── plugins/
│   └── themes/
│
└── decidim/                   ← Nur für Decidim.rs
    ├── plugins/
    └── modules/
```

**Vorteil:** Wiki.rs nutzt `shared/i18n`, `shared/themes` und `wiki/plugins` — alles aus demselben Store. Keine Duplizierung.

```toml
# Store-Config im Programm
[[stores]]
name = "FSN Official"
url = "https://github.com/FreeSynergy/Store"
namespaces = ["shared", "node"]    # Node lädt nur shared + node

# Wiki.rs würde laden:
# namespaces = ["shared", "wiki"]
```

## 6.2 Store-Berechtigungen

| Rolle | Darf im Store |
|---|---|
| **Admin** | Alles installieren/entfernen auf allen Nodes |
| **Node-Admin** | Services auf eigenen Nodes installieren/entfernen |
| **User** | Auf eigenem Desktop: Themes, Sprachen, Widgets ändern |
| **Gast** | Nichts installieren, nur ansehen |

---

# TEIL 7 — WAS ICH (CLAUDE) NOCH ERGÄNZE

## 7.1 Service-Dependency-Graph

Wenn Services installiert werden, zeigt das System den Dependency-Graph:

```
PostgreSQL ──► Kanidm ──► BookStack
                    └──► Forgejo
                    └──► Vikunja
Stalwart ──────────────► BookStack
                    └──► Forgejo (Notifications)
```

Wenn PostgreSQL ausfällt, weiß das System sofort welche Services betroffen sind.

## 7.2 Rollback

Wenn eine Service-Installation fehlschlägt → automatischer Rollback auf den vorherigen Zustand. Quadlet-Dateien werden versioniert. Bei Update: altes Quadlet bleibt als `.bak` bis der neue Service healthy ist.

## 7.3 Dry-Run

```bash
fsn install kanidm --dry-run
# Zeigt was passieren WÜRDE, ohne etwas zu ändern
# → Quadlet-Datei (Preview)
# → Abhängigkeiten die installiert werden
# → Capabilities die verfügbar werden
# → Bus-Events die registriert werden
```

## 7.4 Export/Import

Komplette Node-Konfiguration exportieren und auf einem neuen Node importieren:

```bash
fsn export --output my-node-config.toml
fsn import my-node-config.toml
```

---

# TEIL 8 — CLAUDE-CODE-ARBEITSPAKETE

**Diese Pakete sind so geschnitten, dass sie in Claude Code einzeln abgearbeitet werden können.**

## Paket 0: Repo-Setup
```
AUFGABE: FreeSynergy/Lib Cargo Workspace erstellen
DATEIEN:
  - Cargo.toml (Workspace mit allen fsn-* Members)
  - Jedes fsn-* Crate: Cargo.toml + src/lib.rs (leer, mit doc-Kommentar)
  - .github/workflows/ci.yml (build, test, clippy, fmt)
  - README.md, LICENSE, CLAUDE.md, CHANGELOG.md
ERGEBNIS: Leeres aber kompilierbares Workspace mit CI
```

## Paket 1: fsn-types
```
AUFGABE: Kern-Typen definieren
DATEIEN: fsn-types/src/{lib.rs, resource.rs, host.rs, project.rs, module.rs, 
         permission.rs, type_system.rs, capability.rs, requirement.rs}
TRAITS: ResourceMeta, Capability, Requirement
ENUMS: ResourceType, ContainerPurpose, HostMode, HostStatus, Action, Scope
TESTS: Unit-Tests für alle Typen
DOCS: README.md, #[doc] auf allen pub Items
```

## Paket 2: fsn-error
```
AUFGABE: Error-Handling + Auto-Repair
DATEIEN: fsn-error/src/{lib.rs, repair.rs, validation.rs}
TRAITS: Repairable
ENUMS: RepairAction, RepairOption, ValidationIssue
TESTS: Tests für Repair-Logik
```

## Paket 3: fsn-config
```
AUFGABE: TOML-Config mit Validierung, Auto-Repair, JSON-Schema
DATEIEN: fsn-config/src/{lib.rs, loader.rs, validator.rs, schema.rs, repair.rs}
ABHÄNGIG VON: fsn-types, fsn-error
TESTS: Kaputte TOML-Dateien → Auto-Repair-Tests
```

## Paket 4: fsn-i18n
```
AUFGABE: Snippet-System mit Fluent
DATEIEN: fsn-i18n/src/{lib.rs, loader.rs, lang_meta.rs, fallback.rs, tools.rs}
         fsn-i18n/locales/de/{actions,nouns,status,errors,validation,phrases,time,help,labels,confirmations,notifications}.ftl
         fsn-i18n/locales/en/{same files}
         fsn-i18n/languages.toml (Metadaten: name=English, direction=ltr, ...)
FEATURES: RTL-Support, Fallback-Kette, find_missing(), Plugin-Extension
TESTS: Übersetzungen laden, Fallback, RTL-Detection, fehlende Keys finden
```

## Paket 5: fsn-crypto
```
AUFGABE: Verschlüsselung, Secrets, Tokens, mTLS
DATEIEN: fsn-crypto/src/{lib.rs, age.rs, mtls.rs, tokens.rs, keys.rs}
FEATURES: age-Encryption für "secret"/"password" Typen, Join-Token-Generierung, CA-Management
```

## Paket 6: fsn-db
```
AUFGABE: SeaORM + rusqlite + WriteBuffer
DATEIEN: fsn-db/src/{lib.rs, entities/*, migrations/*, write_buffer.rs}
ENTITIES: resource, permission, sync_state, plugin, audit_log, host, project, module, service_registry
```

## Paket 7: fsn-sync
```
AUFGABE: Automerge CRDT
DATEIEN: fsn-sync/src/{lib.rs, engine.rs, transport.rs, merge.rs, conflict.rs}
```

## Paket 8: fsn-theme
```
AUFGABE: Theme-System
DATEIEN: fsn-theme/src/{lib.rs, loader.rs, css_gen.rs, tui_palette.rs, converter.rs}
         fsn-theme/themes/freesynergy-dark.toml
FEATURES: TOML→CSS, TOML→TUI-Palette, CSS→TOML Import
```

## Paket 9: fsn-pkg + fsn-store
```
AUFGABE: Package-Manager + Store-Client
DATEIEN: fsn-pkg/src/{lib.rs, manifest.rs, capability_match.rs, variable_types.rs, 
         dependency_resolver.rs, installer.rs}
         fsn-store/src/{lib.rs, client.rs, catalog.rs, cache.rs, search.rs}
FEATURES: OCI-kompatibel, Capability-Matching, Variablen-Rollen-System, Namespaces
```

## Paket 10: fsn-plugin-sdk + fsn-plugin-runtime
```
AUFGABE: WASM Plugin System
DATEIEN: fsn-plugin-sdk/src/{lib.rs, traits.rs, wit/}
         fsn-plugin-runtime/src/{lib.rs, host.rs, sandbox.rs}
```

## Paket 11: fsn-bus + fsn-channel + fsn-bot + fsn-llm
```
AUFGABE: Message Bus, Channels, Bots, LLM
DATEIEN: fsn-bus/src/{lib.rs, event.rs, router.rs, transform.rs, buffer.rs}
         fsn-channel/src/{lib.rs, traits.rs, matrix.rs, telegram.rs, normalized.rs}
         fsn-bot/src/{lib.rs, command.rs, registry.rs, commands/}
         fsn-llm/src/{lib.rs, provider.rs, ollama.rs, claude.rs, interpreter.rs}
```

## Paket 12: fsn-federation + fsn-auth
```
AUFGABE: OIDC, SCIM, ActivityPub, OAuth2, JWT
DATEIEN: fsn-federation/src/{lib.rs, oidc.rs, scim.rs, activitypub.rs, webfinger.rs, well_known.rs}
         fsn-auth/src/{lib.rs, oauth2.rs, jwt.rs, rbac.rs, permissions.rs}
```

## Paket 13: fsn-container + fsn-template + fsn-health
```
AUFGABE: Quadlet-Management (OHNE Socket!), Tera, Health-Checks
DATEIEN: fsn-container/src/{lib.rs, quadlet.rs, systemctl.rs, service.rs}
         fsn-template/src/{lib.rs, engine.rs, validator.rs}
         fsn-health/src/{lib.rs, checker.rs, reporter.rs}
```

## Paket 14: fsn-ui
```
AUFGABE: Alle Dioxus-Komponenten
DATEIEN: fsn-ui/src/components/{button,window,form,input,select,table,toast,
         sidebar,status_bar,card,badge,progress,spinner,tabs,scroll_container,
         modal,search_bar,tooltip,context_menu,app_launcher,notification,
         help_panel,theme_switcher,lang_switcher,llm_chat,code_block}.rs
FEATURES: Glassmorphism, Animationen, RTL-Support, TUI-Fallbacks
```

## Paket 15: fsn-help + fsn-bridge-sdk
```
AUFGABE: Hilfe-System + Bridge-Interface
DATEIEN: fsn-help/src/{lib.rs, topic.rs, search.rs, context.rs}
         fsn-bridge-sdk/src/{lib.rs, traits.rs, adapter.rs}
```

## Paket 16: fsn-wizard (Installation)
```
AUFGABE: Installations-Wizard (UI-unabhängig)
DATEIEN: fsn-wizard/src/{lib.rs, steps/network.rs, steps/iam.rs, steps/proxy.rs,
         steps/services.rs, steps/languages.rs, steps/timezone.rs, steps/store.rs,
         join.rs, token.rs, discovery.rs, capability_matcher.rs}
FEATURES: Join-Token-Verifizierung, mDNS-Discovery, Auto-Config von Rollen-Variablen
```

## Paket 17: Node-Anwendung
```
AUFGABE: fsn-node-core, fsn-deploy, fsn-host, fsn-node-cli
ABHÄNGIG VON: Alle fsn-* Libraries
```

## Paket 18: Desktop-Anwendung
```
AUFGABE: fsn-desktop-app, fsn-desktop-views
ABHÄNGIG VON: Alle fsn-* Libraries, fsn-ui
FEATURES: WGUI/GUI/TUI/Web Modi, alle Views
```

---

## Reihenfolge für Claude Code

```
Paket 0  → Repo-Setup
Paket 1  → fsn-types
Paket 2  → fsn-error
Paket 3  → fsn-config
Paket 4  → fsn-i18n (mit allen Snippets de+en)
Paket 5  → fsn-crypto
Paket 6  → fsn-db
Paket 7  → fsn-sync
Paket 8  → fsn-theme
Paket 9  → fsn-pkg + fsn-store
Paket 10 → fsn-plugin-sdk + fsn-plugin-runtime
Paket 11 → fsn-bus + fsn-channel + fsn-bot + fsn-llm
Paket 12 → fsn-federation + fsn-auth
Paket 13 → fsn-container + fsn-template + fsn-health
Paket 14 → fsn-ui
Paket 15 → fsn-help + fsn-bridge-sdk
Paket 16 → fsn-wizard
Paket 17 → Node-Anwendung
Paket 18 → Desktop-Anwendung
```

Jedes Paket kann als eigener Claude-Code-Task abgearbeitet werden. Pakete mit niedrigerer Nummer müssen vor höheren fertig sein (Abhängigkeiten).
