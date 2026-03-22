# FreeSynergy.Node – Complete Rule Set

## Philosophy: Why This Exists

FreeSynergy.Node is built around one core principle: **decentralization with
voluntary cooperation**.

Everyone runs their own instance. Nobody has to trust a central authority.
Nobody gives their data to anyone else. Anyone can install this themselves,
on their own hardware, without asking permission.

At the same time, cooperation is possible – but always opt-in, always
transparent, always revokable. You decide who you work with. You decide
what you share.

This is not just a technical decision. It is the reason the whole system
is designed the way it is.

---

## Core Concept

```
FreeSynergy.Node = Modular, decentralized Podman/Quadlet deployment system
│
├── containers/    Git-tracked: container definitions (TOML + Templates)
├── hosts/         Git-tracked: one file per host (infrastructure layer)
├── projects/      Mixed: project files git-tracked, data/configs git-ignored
│   ├── {name}.project.yml       local deployment (this machine)
│   └── {name}.{hostname}.yml    remote deployment (other machine, same project)
│
└── fsn CLI reads everything -> generates Quadlets -> deploys
```

---

## Directory Structure

```
/opt/FreeSynergy.Node/
├── containers/
│   ├── zentinel/                          # container = directory
│   │   ├── zentinel.toml                  # class file (same name as dir)
│   │   ├── plugins/                       # plugins belong to their container
│   │   │   ├── dns/
│   │   │   │   ├── hetzner.toml
│   │   │   │   └── cloudflare.toml
│   │   │   └── acme/
│   │   │       ├── letsencrypt.toml
│   │   │       └── zerossl.toml
│   │   ├── templates/
│   │   │   └── zentinel.kdl.j2
│   │   └── zentinel-control-plane/
│   │       └── zentinel-control-plane.toml
│   ├── kanidm/
│   │   ├── kanidm.toml
│   │   └── templates/kanidm.toml.j2
│   ├── stalwart/
│   ├── forgejo/
│   ├── outline/
│   ├── cryptpad/
│   ├── postgres/
│   └── dragonfly/
│
├── hosts/
│   └── hetzner-1.host.toml                # one file per host
│
└── projects/
    └── FreeSynergy.Net/
        ├── freesynergy.project.toml       # git-tracked (local containers)
        ├── freesynergy.turbo.toml         # git-tracked (remote host containers)
        ├── freesynergy.federation.toml    # git-tracked (federation config)
        ├── configs/                       # git-ignored (instance secrets)
        └── data/                          # git-ignored (container volumes)
```

---

## File Naming Convention (Projects)

The filename defines the deployment target:

```
{projectname}.project.yml       -> local (this machine)
{projectname}.{hostname}.yml    -> remote (copied to host, executed there)
{projectname}.federation.yml    -> federation config (who can access)
```

Same project name = same project. Different file = different host.
The remote file is copied to the target host and deployed there.
It can load modules and create containers just like a local project file.
No `host:` field needed – the filename is the information.

---

## Host File (Infrastructure Layer)

One file per physical or virtual host. The host file defines what runs
at the OS/infrastructure level, independent of any project.

A host file is ALWAYS required, even for localhost. The installer
creates it automatically during setup.

```yaml
# hosts/hetzner-1.host.yml
host:
  name: "hetzner-1"
  ip: "1.2.3.4"
  ipv6: "2a01::1"               # optional
  external: false               # true = no SSH access, read-only

  proxy:
    zentinel:
      module_class: "zentinel"
      load:
        plugins:
          dns: "hetzner"
          acme: "letsencrypt"
  # Routes auto-collected from ALL project files on this host.
  # No manual list needed.
```

Host files are git-ignored. Only example files are tracked:
```
hosts/
├── example.host.yml              # git-tracked (template)
└── .gitignore                    # ignores everything except example*
```

Key rules:
- `external: false` = fsn CLI can connect and deploy
- `external: true` = no deployment, only config variables are read
- The proxy lives in the host file, not in any project file
- The proxy collects routes automatically from all projects on this host

### Why the Proxy Belongs to the Host

The proxy is bound to an IP address. An IP belongs to a host, not to a
project. Multiple projects share one proxy per host. Therefore the proxy
is defined at host level and collects its routes automatically.

### Proxy KDL Management (Marker-Based)

The proxy KDL config can be modified by two sources: the deployer
(`fsn`) and the Control-Plane (manual/API). To prevent conflicts,
the deployer uses markers to identify its managed sections:

```
# === FSN-MANAGED-START: forgejo ===
forgejo.freesynergy.net {
    reverse_proxy forgejo:3000
}
# === FSN-MANAGED-END: forgejo ===
```

Rules:
- Deployer only touches content between its own markers
- New service → add new marker block
- Changed service → update marker block
- Removed service → delete marker block
- Content outside markers is NEVER touched (Control-Plane owned)
- After changes: Zentinel hot-reload (no restart needed)

---

## Project File Structure

```yaml
# projects/FreeSynergy.Net/freesynergy.project.yml
project:
  name: "FreeSynergy.Net"
  domain: "freesynergy.net"
  description: "..."

  branding:
    logo: "branding/logo.svg"
    favicon: "branding/favicon.ico"
    background: "branding/background.jpg"
    css: "branding/custom.css"
    color_primary: "#1a7f64"
    color_accent: "#22d3ee"

  contact:
    email: "admin@freesynergy.net"
    acme_email: "admin@freesynergy.net"

load:
  modules:
    "kanidm":
      module_class: "kanidm"
    "stalwart":
      module_class: "stalwart"
    "forgejo":
      module_class: "forgejo"
    "outline":
      module_class: "outline"
    "cryptpad":
      module_class: "cryptpad"
```

Branding files live in the project directory (git-tracked):
```
projects/FreeSynergy.Net/
├── branding/                    # git-tracked
│   ├── logo.svg
│   ├── favicon.ico
│   ├── background.jpg
│   └── custom.css
├── freesynergy.project.yml
├── configs/
└── data/
```

Each module decides if and how it uses branding. Modules that support
custom logos, CSS, or colors read from `project.branding`. Modules
that don't support it simply ignore the block.

Kanidm can display logos per federation group or OIDC client.

Rules:
- Only modules running on THIS host belong in `*.project.yml`
- Modules on other hosts go in `{name}.{hostname}.yml`
- No cross-host references inside a single project file
- The proxy is NOT listed here – it lives in the host file

---

## external Flag

Can be set at two levels independently:

```yaml
# Host level: no SSH access at all
host:
  external: true

# Instance level: service exists but is not mine to deploy
load:
  modules:
    their-wiki:
      module_class: "outline"
      external: true    # no container created, only connection vars used
```

Use cases:
- `host.external: true` = server I know about but cannot control
- `instance.external: true` = service someone else runs, I only consume it

---

## DNS Convention

```
Project name  = domain          freesynergy.net
Module name   = subdomain       forgejo.freesynergy.net
Module alias  = CNAME           git.freesynergy.net -> forgejo.freesynergy.net
Host.ip       = A-Record        forgejo.freesynergy.net -> 1.2.3.4
Host.ipv6     = AAAA-Record     forgejo.freesynergy.net -> 2a01::1
```

DNS records are generated automatically. The deployer collects all modules
from all projects on a host and creates DNS entries for each one.

---

## Security: Ports and Network Isolation

### No sudo required

Low ports (80, 443, 25, 587) are handled via kernel parameter:
```
net.ipv4.ip_unprivileged_port_start=80
```
Set once during server setup via `/etc/sysctl.d/`. After that, Podman
can bind any port without root. No sudo, no capabilities, no risk.

Minimum Podman version: 5.0 (Quadlet support + stability).

### Network isolation (MANDATORY)

```
Internet -> Zentinel (80, 443)
                |
                +-- HTTPS (Layer 7) -> services via internal Podman networks
                |
                +-- SMTP/IMAP (Layer 4 TCP) -> stalwart via internal network
```

Rules:
- ONLY Zentinel has external network access
- Zentinel forwards SMTP/IMAP via Layer-4 TCP to Stalwart
- Stalwart has NO published_ports
- All other services: internal networks only, zero external access
- No service may reach the internet except through Zentinel

If a service is compromised, it cannot phone home. The blast radius
of any breach is contained to its local network segment.

### Firewall Management

Firewall ports are managed by the proxy module via deploy/undeploy hooks.
Only the proxy opens external ports. All other services stay internal.

Rules:
- Deploy opens ports: 80/tcp, 443/tcp (always), 25/587/993/tcp (if mail)
- Undeploy closes the same ports
- Only port 22 (SSH) is open by default
- Firewall changes use `firewalld` (permanent + immediate)
- No module other than the proxy may open firewall ports

---

## Data and Configs Location

```
projects/{ProjectName}/
├── configs/{instance_name}/    # git-ignored (secrets + instance config)
└── data/{instance_name}/       # git-ignored (container volumes)
```

- Directory name = instance name (not module class name)
- `data/wiki/` if instance is named "wiki", even if module_class is `outline`
- One `tar` of the project directory = complete backup
- `config_dir` in every module: `{{ project_root }}/data/{{ instance_name }}`

---

## Vault vs Config Variables

```yaml
# vault_ prefix = SECRETS (passwords, tokens, private keys)
vault_db_password             # database password
vault_hetzner_dns_api_token   # API token
vault_outline_secret_key      # application secret

# No prefix = CONFIGURATION (not secret, instance-specific)
outline_domain                # "wiki.freesynergy.net"
log_level                     # "info"
oidc_auth_uri                 # "https://auth.freesynergy.net/..."
```

Rule: `vault_` only when the value must never appear in logs or plain text.

---

## Module Interface (Standard)

### Module vs Service

A **container class** is a template: `kanidm`, `forgejo`. It is a
blueprint. A **service** is a running instance created from a module. It
has a name, a subdomain, a port. The proxy knows services. DNS knows
services. Everything outside of `load:` operates on services.

### load: at Project Level vs Module Level

**Project level** (`*.project.yml`):
Only `load.modules` exists. These are the main programs of the project.
Each entry creates a service. No `load.services` at project level because
nothing is instantiated yet to reference.

Sub-modules (postgres, dragonfly) CAN be loaded at project level if the
service should be shared across multiple modules. In that case, modules
reference the shared instance via `load.services` instead of loading
their own sub-instance.

**Module level** (inside a module class):
- `load.modules` = sub-dependencies. Creates a sub-instance owned by this
  module (e.g. forgejo loads its own postgres).
- `load.services` = config access. Reads another service's variables
  without creating anything. Used when a module needs to know where
  another service lives (e.g. its domain, port, credentials).

The proxy uses `load.services` to discover which services need subdomains
and aliases. If a sub-module of the proxy is not a standalone module, it
is loaded via `load.modules` within the proxy module.

### Service Naming and Network Grouping

In `load.services`, a name prefix can be added. This becomes the network
name: `{prefix}-{servicename}-net`. This allows grouping services into
shared networks when needed.

### Plugins

Plugins are NOT containers. They are helper configurations specific to a
container. Examples: DNS providers, ACME providers for the proxy.
They live in `containers/{name}/plugins/{plugin_type}/`.

### Plugin Convention

Plugins belong to their container directory. For example, zentinel's
DNS and ACME plugins live inside the zentinel container directory.

Directory structure:
```
containers/{name}/plugins/{plugin_type}/{name}.toml
```

Example:
```
containers/zentinel/plugins/
├── dns/
│   ├── hetzner.toml
│   └── cloudflare.toml
└── acme/
    ├── letsencrypt.toml
    └── zerossl.toml
```

#### Plugin Interface

```toml
# containers/zentinel/plugins/dns/hetzner.toml
[plugin]
name        = "hetzner"
type        = "dns"
description = "Hetzner DNS API provider"

[vars]
dns_provider  = "hetzner"
dns_api_url   = "https://dns.hetzner.com/api/v1"
dns_api_token = "{{ vault_dns_api_token }}"
dns_ttl       = 300
```

```toml
# containers/zentinel/plugins/acme/letsencrypt.toml
[plugin]
name        = "letsencrypt"
type        = "acme"
description = "Let's Encrypt ACME provider"

[vars]
acme_provider = "letsencrypt"
acme_ca_url   = "https://acme-v02.api.letsencrypt.org/directory"
acme_email    = "{{ acme_contact_email }}"
```

Field order: `plugin` → `vars`

Plugins have NO `load`, `container`, or `environment` blocks.
They are purely variable collections. The container decides what to do
with the variables in its own templates.

#### Plugin Loading

Plugins are loaded in the host file via `load:` on the proxy:

```yaml
# hosts/hetzner-1.host.toml
host:
  name: "hetzner-1"
  ip: "1.2.3.4"

  proxy:
    zentinel:
      module_class: "zentinel"
      load:
        plugins:
          dns: "hetzner"
          acme: "letsencrypt"
```

The deployer resolves `dns: "hetzner"` to `containers/zentinel/plugins/dns/hetzner.toml`
and loads its vars.

#### Plugin Secrets

Secret values (tokens, keys) are set in the instance config, not in the
plugin file. The plugin references them via `vault_` variables:

```yaml
# Plugin defines the variable name:
vars:
  dns_api_token: "{{ vault_dns_api_token }}"

# Instance config provides the value:
# projects/FreeSynergy.Net/configs/zentinel/zentinel.yml
vault_dns_api_token: "my-secret-token"
```

Different hosts can use the same plugin with different secrets, because
each host has its own instance config directory.

```toml
# containers/{name}/{name}.toml

[module]
name = "{name}"
  alias: []               # optional -> CNAME records
  dns: {}                 # ONLY for type: mail (mx, srv, txt)
  type: "{type}"
  author: "FreeSynergy.Node"
  version: "1.0.0"        # increment to trigger update-stack
  tags: []
  description: "..."
  website: "..."
  repository: "..."
  port: {internal_port}   # always required

  constraints:            # deployer enforces these
    per_host: ~           # max instances per host (null = unlimited)
    per_ip: ~             # max instances per IP
    locality: ~           # "same_host" = must run with consumer

  federation:             # optional, only if module supports federation
    enabled: false        # true = can be used by federated partners
    min_trust: 3          # minimum trust level required (0-4)

vars:
  config_dir: "{{ project_root }}/data/{{ instance_name }}"

load:
  modules: {}             # loads module class -> creates sub-instance (e.g. postgres, dragonfly)
  services: {}            # reads another service's config vars, no container created

container:
  name: "{{ instance_name }}"
  image: "..."
  image_tag: "latest"
  networks: []            # AUTO-GENERATED - never set manually
  volumes:
    - "{{ vars.config_dir }}/data:/data:Z"
  published_ports: []     # FORBIDDEN except proxy

environment:
  SECRET: "{{ vault_secret }}"
  DOMAIN: "{{ service_domain }}"
  LOG_LEVEL: "{{ log_level | default('info') }}"
```

### Field Order (mandatory)
`module` → `vars` → `load` → `container` → `environment`

---

## Container Lifecycle Hooks

Hooks are declared in `[[lifecycle.on_install]]`, `[[lifecycle.on_update]]`, etc.
within the container TOML. See `ServiceLifecycle` in `service/mod.rs` for full spec.

| Hook | Triggered by | When |
|------|-------------|------|
| `on_install` | `fsn deploy` | after container start |
| `on_configure` | `fsn deploy` | during configure phase |
| `on_update` | `fsn update` | after image pull |
| `on_decommission` | `fsn remove` | before removal |

Auto-discovery via glob. File exists = runs. Missing = skipped silently.

---

## Version Management

One Git repo, version in container class:
```toml
[module]
version = "1.2.0"   # increment to trigger update
```

The deployer compares `module.version` vs `instance.deployed_version`.
After deploy, `deployed_version` is written to the instance config.

---

## CLI Commands

| Command | Purpose |
|---------|---------|
| `fsn deploy` | Reconcile desired state, start/update containers |
| `fsn sync` | Show what would change (read-only) |
| `fsn update` | Pull new images + redeploy changed containers |
| `fsn stop` | Stop containers (data retained) |
| `fsn remove` | Run decommission hooks + delete everything |
| `fsn status` | Show running containers and health |

### deploy options
```
fsn deploy                 → all hosts (local + remote)
fsn deploy --local         → only this host
fsn deploy --host turbo    → only host turbo
fsn deploy --service wiki  → only the wiki container
```

### sync logic
```
1. Read all project files: *.project.yml (local) + *.{hostname}.yml (remote)
2. Together = desired state of the entire project
3. Check each host: does actual state match desired state?
4. Report: missing, diverged, extra, ok
5. No changes made (read-only)
```

### deploy logic
```
1. Run sync (compare desired vs actual)
2. For each host:
   - Missing container → deploy
   - Config diverged → update config + restart
   - Extra (not in any project file) → remove
   - Matching → skip
3. Result: entire project consistent across all hosts
```

---

## Available Containers

| Name | Port | Constraints | Notes |
|------|------|-------------|-------|
| `zentinel` | 443 | per_host:1, per_ip:1 | proxy, DNS+ACME plugins |
| `zentinel-control-plane` | 8080 | - | sub-container of zentinel |
| `kanidm` | 8443 | - | IAM / OIDC / LDAP |
| `stalwart` | 443 | - | all-in-one mail server |
| `forgejo` | 3000 | - | Git forge |
| `outline` | 3000 | - | wiki / knowledge base |
| `cryptpad` | 3000 | - | encrypted collab suite |
| `postgres` | 5432 | locality:same_host | relational database |
| `dragonfly` | 6379 | locality:same_host | Redis-compatible cache |
| `vikunja` | 3456 | - | task manager |
| `pretix` | 80 | - | event ticketing |
| `umap` | 8000 | - | OpenStreetMap sharing |
| `openobserver` | 5080 | - | observability platform |
| `otel-collector` | 4318 | per_host:1 | OpenTelemetry collector |
| `tuwunel` | 8448 | - | Matrix homeserver |

---

## Hard Rules

### MUST
```
✅ Container = directory: containers/{name}/{name}.toml
✅ TOML file has same name as directory
✅ config_dir: project_root/data/instance_name
✅ vault_ prefix ONLY for real secrets
✅ module.version always set
✅ module.port always set
✅ module.constraints set for proxy and locality-bound containers
✅ Field order: module → vars → load → container → environment
✅ Host file ALWAYS required, even for localhost
✅ Unique service names per project (duplicate = error, abort)
✅ Proxy defined in host file, not project file
✅ Filename: {name}.project.toml = local, {name}.{host}.toml = remote
✅ Filename: {name}.federation.toml = federation config
✅ Federation provider list must be signed (Ed25519)
✅ net.ipv4.ip_unprivileged_port_start=80 on every host
✅ Plugins inside their container: containers/{name}/plugins/{type}/
✅ Plugin field order: plugin → vars (no load, container, environment)
✅ Plugin secrets via vault_ variables in instance config
✅ Firewall ports opened on deploy, closed on undeploy (proxy only)
✅ Comments in files: English only
✅ Chat: German
```

### FORBIDDEN
```
❌ published_ports on any container (proxy is the only exception)
❌ networks: set manually (auto-generated)
❌ Secrets in GIT
❌ dns: block on non-mail containers
❌ ip in container class
❌ vault_ on non-secret variables
❌ Jinja2 in directory names
❌ Duplicate service names within a project
❌ Proxy in project file
❌ Cross-host container references inside a single project file
❌ Plugins with load, container, or environment blocks
❌ Firewall ports opened by any container other than the proxy
❌ Any service communicating directly with the internet
```

---

## Federation

### Philosophy

Federation = decentralized cooperation between autonomous nodes.
Every node is self-sufficient. No node depends on another to function.
If a node goes down, the rest of the network continues.

Federation is NOT app-level federation (ActivityPub, Matrix, SCIM).
Those are protocols of individual apps. Platform federation controls
WHO gets access to WHICH services, via Kanidm + OIDC.

### Architecture

Every federated node runs its own Kanidm. Nodes trust each other
by accepting OIDC tokens from each other's Kanidm instances.
A signed provider list defines which Kanidm instances are trusted.

```
Node A (kanidm.freesynergy.net)     ──┐
Node B (kanidm.alice.example.org)   ──┼── mutual OIDC trust
Node C (kanidm.carol.net)           ──┘
```

Node A dies → B and C continue. Users registered on A cannot log in
elsewhere, but all other users and services are unaffected.

### Provider List (signed, distributed)

One node publishes the provider list. All other nodes fetch it
periodically. The list is signed with Ed25519. If the signing node
goes down, the next node in priority order takes over.

```yaml
# Published at: https://federation.freesynergy.net/providers.yml
federation_providers:
  version: 5
  updated: "2025-02-27T14:30:00Z"
  signed_by: "ed25519:aaa..."
  signature: "ed25519sig:xyz789..."

  providers:
    - issuer: "https://kanidm.freesynergy.net"
      public_key: "ed25519:aaa..."
      name: "FreeSynergy.Net"
      priority: 1
      status: active

    - issuer: "https://kanidm.alice.example.org"
      public_key: "ed25519:bbb..."
      name: "Alice"
      priority: 2
      status: active
```

### Failover

The `priority` field determines the signing order.
Priority 1 = current signer and publisher of the list.

```
1. Fetch list from source URL
2. URL unreachable? → try next node in priority order
3. All unreachable? → use local copy, keep working
4. Primary down permanently? → priority 2 signs a new list,
   becomes the new primary. All nodes already know its public key.
```

### Invite Flow

Partners join via signed invite tokens (Ed25519).

```
1. Operator generates invite token (signed with federation key)
2. Token contains: partner name, trust level, services, expiry
3. Token sent out-of-band (Signal, email, in person)
4. Partner redeems token at Kanidm
5. Kanidm validates signature → creates group with defined permissions
6. Revoke: deactivate Kanidm group → immediate access loss
```

### Federation File

```yaml
# projects/FreeSynergy.Net/freesynergy.federation.yml

federation:
  name: "FreeSynergy Federation"
  signing_key: "{{ vault_federation_signing_key }}"

  provider_list:
    source: "https://federation.freesynergy.net/providers.yml"
    verify_key: "ed25519:abc123..."
    auto_update: true
    update_interval: 3600
    fallback: local

  trusted_issuers:
    - issuer: "https://kanidm.freesynergy.net"
      public_key: "ed25519:aaa..."
      name: "FreeSynergy.Net"
      priority: 1
      added: "2025-01-01"

    - issuer: "https://kanidm.alice.example.org"
      public_key: "ed25519:bbb..."
      name: "Alice"
      priority: 2
      added: "2025-03-15"

  subprojects:
    "bob":
      subdomain: "bob"
      services: [kanidm, stalwart]
      modules:
        "forgejo":
          module_class: "forgejo"
        "outline":
          module_class: "outline"
      status: active
```

### Trust Levels

```
Level 0: Public      -> open registration
Level 1: Invited     -> invite token required
Level 2: Approved    -> application + manual approval
Level 3: Trusted     -> full access to shared services
Level 4: Sub-project -> runs on my hardware, I am responsible
```

Permissions within a trust level are managed entirely through
Kanidm groups and OIDC claims. The module only declares
`federation.min_trust` – everything else is Kanidm's job.

---

## Special Cases

**CryptPad**: Two proxy entries required (main + sandbox domain).
**Stalwart**: Receives SMTP/IMAP via Zentinel Layer-4 TCP forward. No published_ports.
**Mail reputation**: Stalwart benefits from a dedicated IP.

### Cache Slot Management (Dragonfly/Redis)

Dragonfly (or Redis) provides 16 database slots (0-15) per instance.
Each service that needs a cache gets one or more slots assigned.

Rules:
- The deployer counts how many `cache_slot_*` variables a module
  references in its `environment:` block
- Slots are assigned sequentially (0, 1, 2, ...) across all services
  on a host that need cache
- When all 16 slots are used, a new cache instance is created
  automatically: `dragonfly`, `dragonfly-2`, `dragonfly-3`, etc.
- Cache is ephemeral – slot assignment does not need to be stable
  across re-deploys
- Each service gets a unique combination of instance + slot
- Some services need 2 slots, some need 0 – determined by environment vars

Example in a module:
```yaml
environment:
  REDIS_URL: "redis://:{{ vault_cache_password }}@{{ cache_host }}:{{ cache_port }}/{{ cache_slot_0 }}"
  REDIS_CACHE_URL: "redis://:{{ vault_cache_password }}@{{ cache_host }}:{{ cache_port }}/{{ cache_slot_1 }}"
```

The deployer resolves `cache_host`, `cache_port`, `cache_slot_0`,
`cache_slot_1` automatically based on which instance has free slots.

---

## Start Command (new chat)

```
Read RULES.md and continue building the FreeSynergy.Node.

Current state:
- Philosophy: decentralized, self-hosted, voluntary cooperation
- Structure: containers/ + hosts/ + projects/
- Deployment: Rust CLI (fsn), Podman Quadlets — no Ansible
- File convention: {name}.project.toml = local, {name}.{host}.toml = remote
- Proxy (zentinel) lives in host file, not project file
- external: flag on host or instance level
- Security: net.ipv4.ip_unprivileged_port_start=80, no published_ports except proxy
- All services isolated: only Zentinel has external access
- Zentinel forwards SMTP/IMAP via Layer-4 TCP to Stalwart
- Containers: zentinel (+control-plane, plugins), kanidm, stalwart, forgejo,
  outline, cryptpad, postgres, dragonfly, vikunja, pretix, umap,
  openobserver, otel-collector, tuwunel, mistral
- Container constraints: per_host, per_ip, locality (enforced by deployer)
- DNS: projectname=domain, containername=subdomain, alias=CNAME
- Federation: signed provider list, OIDC trust between nodes,
  priority-based failover, invite tokens (Ed25519 signed)

Next step: {describe what to do next}
```
