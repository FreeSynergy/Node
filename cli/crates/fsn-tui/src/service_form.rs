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

// ── No-op change hook ─────────────────────────────────────────────────────────

fn service_on_change(_form: &mut ResourceForm, _idx: usize) {
    // Future: auto-derive subdomain from instance name, etc.
}

// ── Form builder ──────────────────────────────────────────────────────────────

/// New (empty) service form.
pub fn new_service_form() -> ResourceForm {
    let fields = vec![
        // ── Tab 0: Service ────────────────────────────────────────────────────
        FormField::new("name",  "form.service.name",  0, true,  FormFieldType::Text)
            .hint("form.service.name.hint"),
        FormField::new("class", "form.service.class", 0, true,  FormFieldType::Select)
            .opts(SERVICE_CLASSES.to_vec())
            .default_val(SERVICE_CLASSES[0])
            .display(service_class_display),
        FormField::new("alias", "form.service.alias", 0, false, FormFieldType::Text)
            .hint("form.service.alias.hint"),
        // ── Tab 1: Options ────────────────────────────────────────────────────
        FormField::new("version", "form.options.version", 1, false, FormFieldType::Text)
            .default_val("latest"),
    ];
    ResourceForm::new(ResourceKind::Service, SERVICE_TABS, fields, None, service_on_change)
}
