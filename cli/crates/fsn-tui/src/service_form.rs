// Service-specific form builders.
//
// Adding this module required zero changes to the generic form machinery —
// that's the whole point of Phase 2.

use crate::app::{FormField, FormFieldType, ResourceForm, ResourceKind, SERVICE_TABS};

// ── Available service classes ─────────────────────────────────────────────────

pub const SERVICE_CLASSES: &[&str] = &[
    "git/forgejo",
    "iam/kanidm",
    "mail/stalwart",
    "wiki/outline",
    "chat/matrix",
    "tasks/vikunja",
    "monitoring/netdata",
];

pub fn service_class_display(code: &str) -> &'static str {
    match code {
        "git/forgejo"        => "Forgejo (Git)",
        "iam/kanidm"         => "Kanidm (IAM)",
        "mail/stalwart"      => "Stalwart (Mail)",
        "wiki/outline"       => "Outline (Wiki)",
        "chat/matrix"        => "Matrix (Chat)",
        "tasks/vikunja"      => "Vikunja (Tasks)",
        "monitoring/netdata" => "Netdata (Monitoring)",
        _                    => "—",
    }
}

// ── Change hook ───────────────────────────────────────────────────────────────

fn service_on_change(form: &mut ResourceForm, idx: usize) {
    if form.fields[idx].key == "name" {
        let slug = crate::app::slugify(&form.fields[idx].value.clone());
        if let Some(s) = form.fields.iter().position(|f| f.key == "subdomain") {
            if !form.fields[s].dirty {
                let len = slug.len();
                form.fields[s].value  = slug;
                form.fields[s].cursor = len;
            }
        }
    }
}

// ── Form builder ──────────────────────────────────────────────────────────────

/// New (empty) service form.
pub fn new_service_form() -> ResourceForm {
    let fields = vec![
        // ── Tab 0: Service ────────────────────────────────────────────────────
        FormField::new("name",      "form.service.name",      0, true,  FormFieldType::Text)
            .hint("form.service.name.hint"),
        FormField::new("class",     "form.service.class",     0, true,  FormFieldType::Select)
            .opts(SERVICE_CLASSES.to_vec())
            .default_val(SERVICE_CLASSES[0])
            .display(service_class_display),
        FormField::new("subdomain", "form.service.subdomain", 0, false, FormFieldType::Text)
            .hint("form.service.subdomain.hint"),
        FormField::new("alias",     "form.service.alias",     0, false, FormFieldType::Text)
            .hint("form.service.alias.hint"),
        // ── Tab 1: Options ────────────────────────────────────────────────────
        FormField::new("version", "form.options.version", 1, false, FormFieldType::Text)
            .default_val("latest"),
        FormField::new("port",    "form.service.port",    1, false, FormFieldType::Text),
    ];
    ResourceForm::new(ResourceKind::Service, SERVICE_TABS, fields, None, service_on_change)
}
