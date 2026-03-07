// Project-specific form builders.
//
// Decoupled from the generic ResourceForm machinery in app.rs.
// Adding a new resource type = create a similar module, zero duplication.

use std::path::Path;

use anyhow::Result;

use crate::app::{FormField, FormFieldType, ProjectHandle, ResourceForm, ResourceKind, PROJECT_TABS};

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

/// Called by `ResourceForm.on_change` after any field edit.
/// Derives domain from name (slug), and contact_email from domain.
pub fn project_on_change(form: &mut ResourceForm, idx: usize) {
    match form.fields[idx].key {
        "name" => {
            let slug = crate::app::slugify(&form.fields[idx].value.clone());
            if let Some(d) = form.fields.iter().position(|f| f.key == "domain") {
                if !form.fields[d].dirty {
                    let len = slug.len();
                    form.fields[d].value  = slug;
                    form.fields[d].cursor = len;
                }
            }
            sync_email_from_domain(form);
        }
        "domain" => sync_email_from_domain(form),
        _ => {}
    }
}

fn sync_email_from_domain(form: &mut ResourceForm) {
    let domain = form.fields.iter()
        .find(|f| f.key == "domain")
        .map(|f| f.value.clone())
        .unwrap_or_default();
    if domain.is_empty() { return; }
    if let Some(e) = form.fields.iter().position(|f| f.key == "contact_email") {
        if !form.fields[e].dirty {
            let email = format!("admin@{}", domain);
            let len   = email.len();
            form.fields[e].value  = email;
            form.fields[e].cursor = len;
        }
    }
}

// ── Form builders ─────────────────────────────────────────────────────────────

/// New (empty) project form.
pub fn new_project_form() -> ResourceForm {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/home/user".into());
    let fields = vec![
        // ── Tab 0: Project ────────────────────────────────────────────────────
        FormField::new("name",          "form.project.name",        0, true,  FormFieldType::Text)
            .hint("form.project.name.hint"),
        FormField::new("domain",        "form.project.domain",      0, true,  FormFieldType::Text)
            .hint("form.project.domain.hint"),
        FormField::new("description",   "form.project.description", 0, false, FormFieldType::Text)
            .hint("form.project.description.hint"),
        FormField::new("contact_email", "form.project.email",       0, true,  FormFieldType::Email)
            .hint("form.project.email.hint"),
        // ── Tab 1: Options ────────────────────────────────────────────────────
        FormField::new("language", "form.options.language",  1, false, FormFieldType::Select)
            .opts(vec!["de", "en", "fr", "es", "it", "pt"])
            .default_val("de")
            .display(lang_display),
        FormField::new("path",     "form.project.path",      1, true,  FormFieldType::Path)
            .default_val(&format!("{}/fsn", home))
            .hint("form.project.path.hint"),
        FormField::new("version",  "form.options.version",   1, false, FormFieldType::Text)
            .default_val("0.1.0"),
    ];
    ResourceForm::new(ResourceKind::Project, PROJECT_TABS, fields, None, project_on_change)
}

/// Edit form pre-filled from an existing project.
pub fn edit_project_form(handle: &ProjectHandle) -> ResourceForm {
    let p    = &handle.config.project;
    let desc = p.description.as_deref().unwrap_or("");
    let fields = vec![
        FormField::new("name",          "form.project.name",        0, true,  FormFieldType::Text)
            .hint("form.project.name.hint").default_val(&p.name).dirty(),
        FormField::new("domain",        "form.project.domain",      0, true,  FormFieldType::Text)
            .hint("form.project.domain.hint").default_val(&p.domain).dirty(),
        FormField::new("description",   "form.project.description", 0, false, FormFieldType::Text)
            .hint("form.project.description.hint").default_val(desc).dirty(),
        FormField::new("contact_email", "form.project.email",       0, true,  FormFieldType::Email)
            .hint("form.project.email.hint").default_val(handle.email()).dirty(),
        FormField::new("language", "form.options.language",  1, false, FormFieldType::Select)
            .opts(vec!["de", "en", "fr", "es", "it", "pt"])
            .default_val(&p.language).dirty()
            .display(lang_display),
        FormField::new("path",     "form.project.path",      1, true,  FormFieldType::Path)
            .hint("form.project.path.hint").default_val(handle.install_dir()).dirty(),
        FormField::new("version",  "form.options.version",   1, false, FormFieldType::Text)
            .default_val(&p.version).dirty(),
    ];
    ResourceForm::new(ResourceKind::Project, PROJECT_TABS, fields, Some(handle.slug.clone()), project_on_change)
}

// ── Submit ────────────────────────────────────────────────────────────────────

/// Write the project form to disk as `projects/{slug}/{slug}.project.toml`.
pub fn submit_project_form(form: &ResourceForm, root: &Path) -> Result<()> {
    let is_edit = form.edit_id.is_some();
    let slug = form.edit_id.clone()
        .unwrap_or_else(|| crate::app::slugify(&form.field_value("name")));

    let project_dir = root.join("projects").join(&slug);
    std::fs::create_dir_all(&project_dir)?;

    let toml_path = project_dir.join(format!("{}.project.toml", slug));
    if !is_edit && toml_path.exists() { return Ok(()); }  // don't overwrite on new

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
