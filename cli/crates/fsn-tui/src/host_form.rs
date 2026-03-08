// Host-specific form builder.
//
// Follows the same pattern as project_form.rs and service_form.rs.
// Fields per tab:
//   Tab 0 (Host)   : name, alias, address, project
//   Tab 1 (System) : ssh_user, ssh_port, install_dir
//   Tab 2 (DNS/TLS): dns_provider, acme_provider, acme_email

use std::path::Path;

use anyhow::Result;

use crate::app::{FormField, FormFieldType, HOST_TABS, ResourceForm, ResourceKind};

// ── Provider lists ─────────────────────────────────────────────────────────────

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

// ── Change hook ───────────────────────────────────────────────────────────────

fn host_on_change(form: &mut ResourceForm, idx: usize) {
    // Auto-derive acme_email from address when address looks like a domain
    if form.fields[idx].key == "address" {
        let addr = form.fields[idx].value.clone();
        if addr.contains('.') && !addr.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            if let Some(e) = form.fields.iter().position(|f| f.key == "acme_email") {
                if !form.fields[e].dirty {
                    let email = format!("admin@{}", addr);
                    let len   = email.len();
                    form.fields[e].value  = email;
                    form.fields[e].cursor = len;
                }
            }
        }
    }
}

// ── Form builders ─────────────────────────────────────────────────────────────

/// New (empty) host form, pre-filled with the owning project slug.
pub fn new_host_form(project_slug: &str) -> ResourceForm {
    let fields = vec![
        // ── Tab 0: Host ────────────────────────────────────────────────────────
        FormField::new("name",    "form.host.name",    0, true,  FormFieldType::Text)
            .hint("form.host.name.hint"),
        FormField::new("alias",   "form.host.alias",   0, false, FormFieldType::Text)
            .hint("form.host.alias.hint"),
        FormField::new("address", "form.host.address", 0, true,  FormFieldType::Ip)
            .hint("form.host.address.hint"),
        FormField::new("project", "form.host.project", 0, false, FormFieldType::Text)
            .default_val(project_slug),
        // ── Tab 1: System ──────────────────────────────────────────────────────
        FormField::new("ssh_user",    "form.host.ssh_user",    1, false, FormFieldType::Text)
            .default_val("root"),
        FormField::new("ssh_port",    "form.host.ssh_port",    1, false, FormFieldType::Text)
            .default_val("22"),
        FormField::new("install_dir", "form.host.install_dir", 1, false, FormFieldType::Path)
            .hint("form.host.install_dir.hint")
            .default_val("/opt/fsn"),
        // ── Tab 2: DNS / TLS ───────────────────────────────────────────────────
        FormField::new("dns_provider",  "form.host.dns_provider",  2, false, FormFieldType::Select)
            .opts(DNS_PROVIDERS.to_vec())
            .default_val(DNS_PROVIDERS[0])
            .display(dns_provider_display),
        FormField::new("acme_provider", "form.host.acme_provider", 2, false, FormFieldType::Select)
            .opts(ACME_PROVIDERS.to_vec())
            .default_val(ACME_PROVIDERS[0])
            .display(acme_provider_display),
        FormField::new("acme_email", "form.host.acme_email", 2, false, FormFieldType::Email)
            .hint("form.host.acme_email.hint"),
    ];
    ResourceForm::new(ResourceKind::Host, HOST_TABS, fields, None, host_on_change)
}

// ── Submit ────────────────────────────────────────────────────────────────────

/// Write the host config to `{project_dir}/{slug}.host.toml`.
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

    let ssh_user_val    = if ssh_user.is_empty()    { "root".to_string() } else { ssh_user };
    let ssh_port_val: u16 = ssh_port.parse().unwrap_or(22);
    let install_dir_val = if install_dir.is_empty() { "/opt/fsn".to_string() } else { install_dir };

    let slug = crate::app::slugify(&name);
    let path = project_dir.join(format!("{}.host.toml", slug));

    // Write TOML manually — same pattern as project_form.rs
    let mut content = format!(
        "[host]\nname        = \"{name}\"\naddress     = \"{address}\"\n"
    );
    if !alias.is_empty() {
        content.push_str(&format!("alias       = \"{alias}\"\n"));
    }
    if !project.is_empty() {
        content.push_str(&format!("project     = \"{project}\"\n"));
    }
    content.push_str(&format!(
        "ssh_user    = \"{ssh_user_val}\"\nssh_port    = {ssh_port_val}\ninstall_dir = \"{install_dir_val}\"\n"
    ));

    if dns_prov != "none" && !dns_prov.is_empty() {
        content.push_str(&format!(
            "\n[dns]\nprovider = \"{dns_prov}\"\nzones    = []\n"
        ));
    }

    if acme_prov != "none" && !acme_prov.is_empty() {
        let email = if acme_email.is_empty() { format!("admin@{}", address) } else { acme_email };
        content.push_str(&format!(
            "\n[acme]\nemail    = \"{email}\"\nprovider = \"{acme_prov}\"\n"
        ));
    }

    std::fs::write(&path, content)?;
    Ok(())
}
