# FreeSynergy · Node

**Modular, decentralized self-hosting stack**  
Rootless · Podman Quadlets · Rust CLI + TUI · Free cooperation between sovereign nodes

FreeSynergy.Node is a **self-sovereign deployment system** for self-hosted services. It runs fully **rootless** on any Linux server, uses **Podman Quadlets** instead of Docker, and operates without any central control plane.

Core tools: native **Rust CLI** (`fsn`), interactive setup wizard, and an evolving **TUI dashboard** (built with ratatui + rat-salsa for clean event-loop, focus handling, and composable widgets).

Vision: Nodes that **voluntarily federate** — with OIDC-based identity sharing in progress.

## Current Status (March 2026)

- **CLI (`fsn`)**: Fully usable (deploy, status, edit, wizard, i18n support)
- **TUI Dashboard**: Under heavy active development — Sidebar, project list, edit/delete actions, toast notifications, sidebar filters, improved keyboard navigation, better errors & tests
- **Setup Wizard**: Complete (module-driven, interactive)
- **Modules**: 14+ services ready (Proxy, IAM, Mail, Git, Nextcloud, Matrix, …)
- **Federation**: OIDC concept & skeleton in place — implementation ongoing
- **Version**: v0.1.0-dev (no stable releases yet)

## Key Features

- **Rootless & secure by default**  
  Podman + Quadlets → no root privileges required  
  Secrets managed via `age`-encrypted vault  
  Internal networks, git-ignored host files

- **Modular & extensible**  
  Services defined via TOML + Jinja2 templates in `modules/`  
  Process-based module plugins (new!)  
  Automatic Quadlet generation + Zentinel reverse-proxy config

- **CLI & TUI**  
  `fsn` — Rust-native command-line tool with subcommands  
  Interactive wizard for first-time setup  
  **TUI dashboard** powered by ratatui + rat-salsa (sidebar navigation, project lists, forms, toasts, filters)

- **Automation**  
  Let's Encrypt TLS certificates  
  Hetzner DNS reconciliation  
  Planned: automatic container updates & rollbacks

- **Decentralized & federated (long-term goal)**  
  No central server  
  Voluntary node cooperation  
  Shared identities via OIDC federation (WIP)

## Quick Start

```bash
# 1. Run the installer (sets up Podman + builds fsn)
curl -sSL https://raw.githubusercontent.com/FreeSynergy/Node/master/fs-install.sh | bash

# 2. Initialize your first node
fsn wizard

# 3. Launch the TUI dashboard (once built)
fsn tui
```

## Project Structure (High-Level)

```
.
├── cli/               # Rust CLI + TUI (fsn binary + fs-tui logic)
├── hosts/             # Per-server configs (gitignore'd)
├── locales/           # i18n files (de + en via Fluent)
├── modules/           # Service definitions (TOML + Jinja templates)
├── projects/          # Your deployed project instances
├── store/             # Bundled store integration
├── tools/             # Build & setup scripts
├── fs-install.sh     # One-shot bootstrap installer
└── ...                # CHANGELOG.md · TODO.md · RULES.md · LICENSE
```

## Tech Stack (OOP-oriented Rust code)

- **Rust** (90%+) — strong OOP patterns: traits for widgets/screens, state encapsulation, delegation to components, builder patterns for complex init
- **ratatui** + **rat-salsa** — modern TUI rendering, fair event polling, focus management, background tasks, composable widgets
- **Podman Quadlets** — declarative container units
- **Jinja2** — templating for units & configs
- **age** — secret encryption
- **i18n** — Fluent-based (German + English)

## Contributing

Project is early-stage → contributions very welcome!

- Open issues & PRs for anything
- Discussions around federation, new modules, TUI polish
- Coding style: Prefer OOP (encapsulation, traits for extensibility, clear delegation)

See `RULES.md` and `CLAUDE.md` for workflow & guidelines.

## License

[MIT License](LICENSE) — with note for future federation-adjusted terms.

---

**FreeSynergy.Node** — because every node should be free.
