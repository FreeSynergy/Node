// Project-specific form builders.
//
// Uses the component-based FormNode system:
//   TextInputNode — text/email/path fields
//   SelectInputNode — language picker
//
// on_change hook derives domain from name and contact_email from domain.

use std::path::Path;

use anyhow::Result;

use crate::app::{ProjectHandle, ResourceForm, ResourceKind, PROJECT_TABS};
use crate::ui::form_node::FormNode;
use crate::ui::nodes::{SelectInputNode, TextInputNode};

// ── Display helper ────────────────────────────────────────────────────────────

pub fn lang_display(code: &str) -> &'static str {
    match code {
        "de" => "Deutsch",
        "en" => "English",
        "fr" => "Français",
        "es" => "Español",
        "it" => "Italiano",
        "pt" => "Português",
        _    => "—",
    }
}

// ── Smart-defaults hook ───────────────────────────────────────────────────────

pub fn project_on_change(nodes: &mut Vec<Box<dyn FormNode>>, key: &'static str) {
    match key {
        "name" => {
            let name_val = nodes.iter().find(|n| n.key() == "name")
                .map(|n| n.value().to_string()).unwrap_or_default();
            let slug = crate::app::slugify(&name_val);

            // Derive domain from slug if not dirty
            let domain_dirty = nodes.iter().find(|n| n.key() == "domain")
                .map(|n| n.is_dirty()).unwrap_or(false);
            if !domain_dirty {
                if let Some(n) = nodes.iter_mut().find(|n| n.key() == "domain") {
                    n.set_value(&slug);
                }
            }

            sync_email_from_domain(nodes);
        }
        "domain" => sync_email_from_domain(nodes),
        _ => {}
    }
}

fn sync_email_from_domain(nodes: &mut Vec<Box<dyn FormNode>>) {
    let domain = nodes.iter().find(|n| n.key() == "domain")
        .map(|n| n.value().to_string()).unwrap_or_default();
    if domain.is_empty() { return; }
    let email_dirty = nodes.iter().find(|n| n.key() == "contact_email")
        .map(|n| n.is_dirty()).unwrap_or(false);
    if !email_dirty {
        let email = format!("admin@{}", domain);
        if let Some(n) = nodes.iter_mut().find(|n| n.key() == "contact_email") {
            n.set_value(&email);
        }
    }
}

// ── Form builders ─────────────────────────────────────────────────────────────

pub fn new_project_form() -> ResourceForm {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".into());
    let nodes: Vec<Box<dyn FormNode>> = vec![
        // ── Tab 0: Project ─────────────────────────────────────────────────
        Box::new(TextInputNode::new("name",          "form.project.name",        0, true)
            .hint("form.project.name.hint")),
        Box::new(TextInputNode::new("domain",        "form.project.domain",      0, true)
            .hint("form.project.domain.hint")),
        Box::new(TextInputNode::new("description",   "form.project.description", 0, false)
            .hint("form.project.description.hint")),
        Box::new(TextInputNode::new("contact_email", "form.project.email",       0, true)
            .hint("form.project.email.hint")),
        // ── Tab 1: Options ─────────────────────────────────────────────────
        Box::new(SelectInputNode::new("language", "form.options.language", 1, false,
            vec!["de", "en", "fr", "es", "it", "pt"])
            .default_val("de")
            .display(lang_display)),
        Box::new(TextInputNode::new("path",    "form.project.path",    1, true)
            .default_val(&format!("{}/fsn", home))
            .hint("form.project.path.hint")),
        Box::new(TextInputNode::new("version", "form.options.version", 1, false)
            .default_val("0.1.0")),
    ];
    ResourceForm::new(ResourceKind::Project, PROJECT_TABS, nodes, None, project_on_change)
}

pub fn edit_project_form(handle: &ProjectHandle) -> ResourceForm {
    let p    = &handle.config.project;
    let desc = p.description.as_deref().unwrap_or("");
    let nodes: Vec<Box<dyn FormNode>> = vec![
        Box::new(TextInputNode::new("name",          "form.project.name",        0, true)
            .hint("form.project.name.hint").pre_filled(&p.name)),
        Box::new(TextInputNode::new("domain",        "form.project.domain",      0, true)
            .hint("form.project.domain.hint").pre_filled(&p.domain)),
        Box::new(TextInputNode::new("description",   "form.project.description", 0, false)
            .hint("form.project.description.hint").pre_filled(desc)),
        Box::new(TextInputNode::new("contact_email", "form.project.email",       0, true)
            .hint("form.project.email.hint").pre_filled(handle.email())),
        Box::new(SelectInputNode::new("language", "form.options.language", 1, false,
            vec!["de", "en", "fr", "es", "it", "pt"])
            .default_val(&p.language)
            .display(lang_display)),
        Box::new(TextInputNode::new("path",    "form.project.path",    1, true)
            .hint("form.project.path.hint").pre_filled(handle.install_dir())),
        Box::new(TextInputNode::new("version", "form.options.version", 1, false)
            .pre_filled(&p.version)),
    ];
    ResourceForm::new(ResourceKind::Project, PROJECT_TABS, nodes, Some(handle.slug.clone()), project_on_change)
}

// ── Submit ────────────────────────────────────────────────────────────────────

pub fn submit_project_form(form: &ResourceForm, root: &Path) -> Result<()> {
    let is_edit = form.edit_id.is_some();
    let slug = form.edit_id.clone()
        .unwrap_or_else(|| crate::app::slugify(&form.field_value("name")));

    let project_dir = root.join("projects").join(&slug);
    std::fs::create_dir_all(&project_dir)?;

    let toml_path = project_dir.join(format!("{}.project.toml", slug));
    if !is_edit && toml_path.exists() { return Ok(()); }

    let name    = form.field_value("name");
    let domain  = form.field_value("domain");
    let desc    = form.field_value("description");
    let email   = form.field_value("contact_email");
    let lang    = form.field_value("language");
    let path    = form.field_value("path");
    let version = form.field_value("version");

    let content = format!(
        "[project]\nname        = \"{name}\"\ndomain      = \"{domain}\"\ndescription = \"{desc}\"\nemail       = \"{email}\"\nlanguage    = \"{lang}\"\ninstall_dir = \"{path}\"\nversion     = \"{version}\"\n"
    );
    std::fs::write(&toml_path, content)?;
    Ok(())
}
