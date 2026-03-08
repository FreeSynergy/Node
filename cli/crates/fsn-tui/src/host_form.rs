// Host-specific form builder.
//
// Fields per tab:
//   Tab 0 (Host)   : name, alias, address, project
//   Tab 1 (System) : ssh_user, ssh_port, install_dir
//   Tab 2 (DNS/TLS): dns_provider, acme_provider, acme_email

use std::path::Path;

use anyhow::Result;

use crate::app::{HOST_TABS, ResourceForm, ResourceKind};
use crate::ui::form_node::FormNode;
use crate::ui::nodes::{SelectInputNode, TextInputNode};

pub const DNS_PROVIDERS:  &[&str] = &["hetzner", "cloudflare", "manual", "none"];
pub const ACME_PROVIDERS: &[&str] = &["letsencrypt", "zerossl", "buypass", "none"];

pub fn dns_provider_display(code: &str) -> &'static str {
    match code {
        "hetzner"    => "Hetzner DNS",
        "cloudflare" => "Cloudflare",
        "manual"     => "Manual",
        "none"       => "None (disabled)",
        _            => "—",
    }
}

pub fn acme_provider_display(code: &str) -> &'static str {
    match code {
        "letsencrypt" => "Let's Encrypt",
        "zerossl"     => "ZeroSSL",
        "buypass"     => "Buypass",
        "none"        => "None (disabled)",
        _             => "—",
    }
}

// ── Smart-defaults hook ───────────────────────────────────────────────────────

fn host_on_change(nodes: &mut Vec<Box<dyn FormNode>>, key: &'static str) {
    if key == "address" {
        let addr = nodes.iter().find(|n| n.key() == "address")
            .map(|n| n.value().to_string()).unwrap_or_default();
        // Only auto-derive if address looks like a domain (not a raw IP)
        if addr.contains('.') && !addr.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            let acme_dirty = nodes.iter().find(|n| n.key() == "acme_email")
                .map(|n| n.is_dirty()).unwrap_or(false);
            if !acme_dirty {
                let email = format!("admin@{}", addr);
                if let Some(n) = nodes.iter_mut().find(|n| n.key() == "acme_email") {
                    n.set_value(&email);
                }
            }
        }
    }
}

// ── Form builder ──────────────────────────────────────────────────────────────

pub fn new_host_form(project_slug: &str) -> ResourceForm {
    let nodes: Vec<Box<dyn FormNode>> = vec![
        // ── Tab 0: Host ───────────────────────────────────────────────────
        Box::new(TextInputNode::new("name",    "form.host.name",    0, true)
            .hint("form.host.name.hint")),
        Box::new(TextInputNode::new("alias",   "form.host.alias",   0, false)
            .hint("form.host.alias.hint")),
        Box::new(TextInputNode::new("address", "form.host.address", 0, true)
            .hint("form.host.address.hint")),
        Box::new(TextInputNode::new("project", "form.host.project", 0, false)
            .default_val(project_slug)),
        // ── Tab 1: System ─────────────────────────────────────────────────
        Box::new(TextInputNode::new("ssh_user",    "form.host.ssh_user",    1, false)
            .default_val("root")),
        Box::new(TextInputNode::new("ssh_port",    "form.host.ssh_port",    1, false)
            .default_val("22")),
        Box::new(TextInputNode::new("install_dir", "form.host.install_dir", 1, false)
            .hint("form.host.install_dir.hint")
            .default_val("/opt/fsn")),
        // ── Tab 2: DNS / TLS ──────────────────────────────────────────────
        Box::new(SelectInputNode::new("dns_provider", "form.host.dns_provider", 2, false,
            DNS_PROVIDERS.to_vec())
            .default_val(DNS_PROVIDERS[0])
            .display(dns_provider_display)),
        Box::new(SelectInputNode::new("acme_provider", "form.host.acme_provider", 2, false,
            ACME_PROVIDERS.to_vec())
            .default_val(ACME_PROVIDERS[0])
            .display(acme_provider_display)),
        Box::new(TextInputNode::new("acme_email", "form.host.acme_email", 2, false)
            .hint("form.host.acme_email.hint")),
    ];
    ResourceForm::new(ResourceKind::Host, HOST_TABS, nodes, None, host_on_change)
}

// ── Submit ────────────────────────────────────────────────────────────────────

pub fn submit_host_form(form: &ResourceForm, project_dir: &Path) -> Result<()> {
    let name    = form.field_value("name");
    let alias   = form.field_value("alias");
    let address = form.field_value("address");
    let project = form.field_value("project");

    if name.is_empty()    { anyhow::bail!("Hostname ist erforderlich"); }
    if address.is_empty() { anyhow::bail!("IP-Adresse / FQDN ist erforderlich"); }

    let ssh_user    = form.field_value("ssh_user");
    let ssh_port    = form.field_value("ssh_port");
    let install_dir = form.field_value("install_dir");
    let dns_prov    = form.field_value("dns_provider");
    let acme_prov   = form.field_value("acme_provider");
    let acme_email  = form.field_value("acme_email");

    let ssh_user_val      = if ssh_user.is_empty()    { "root".to_string()    } else { ssh_user };
    let ssh_port_val: u16 = ssh_port.parse().unwrap_or(22);
    let install_dir_val   = if install_dir.is_empty() { "/opt/fsn".to_string() } else { install_dir };

    let slug = crate::app::slugify(&name);
    let path = project_dir.join(format!("{}.host.toml", slug));

    let mut content = format!(
        "[host]\nname        = \"{name}\"\naddress     = \"{address}\"\n"
    );
    if !alias.is_empty()   { content.push_str(&format!("alias       = \"{alias}\"\n")); }
    if !project.is_empty() { content.push_str(&format!("project     = \"{project}\"\n")); }
    content.push_str(&format!(
        "ssh_user    = \"{ssh_user_val}\"\nssh_port    = {ssh_port_val}\ninstall_dir = \"{install_dir_val}\"\n"
    ));

    if dns_prov != "none" && !dns_prov.is_empty() {
        content.push_str(&format!("\n[dns]\nprovider = \"{dns_prov}\"\nzones    = []\n"));
    }
    if acme_prov != "none" && !acme_prov.is_empty() {
        let email = if acme_email.is_empty() { format!("admin@{}", address) } else { acme_email };
        content.push_str(&format!("\n[acme]\nemail    = \"{email}\"\nprovider = \"{acme_prov}\"\n"));
    }

    std::fs::write(&path, content)?;
    Ok(())
}
