// Service-specific form builders.

use crate::app::{ResourceForm, ResourceKind, SERVICE_TABS};
use crate::ui::form_node::FormNode;
use crate::ui::nodes::{SelectInputNode, TextInputNode};

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

// ── Smart-defaults hook ───────────────────────────────────────────────────────

fn service_on_change(nodes: &mut Vec<Box<dyn FormNode>>, key: &'static str) {
    if key == "name" {
        let name_val = nodes.iter().find(|n| n.key() == "name")
            .map(|n| n.value().to_string()).unwrap_or_default();
        let slug = crate::app::slugify(&name_val);

        let subdomain_dirty = nodes.iter().find(|n| n.key() == "subdomain")
            .map(|n| n.is_dirty()).unwrap_or(false);
        if !subdomain_dirty {
            if let Some(n) = nodes.iter_mut().find(|n| n.key() == "subdomain") {
                n.set_value(&slug);
            }
        }
    }
}

// ── Form builder ──────────────────────────────────────────────────────────────

pub fn new_service_form() -> ResourceForm {
    let nodes: Vec<Box<dyn FormNode>> = vec![
        // ── Tab 0: Service ─────────────────────────────────────────────────
        Box::new(TextInputNode::new("name",      "form.service.name",      0, true)
            .hint("form.service.name.hint")),
        Box::new(SelectInputNode::new("class", "form.service.class", 0, true,
            SERVICE_CLASSES.to_vec())
            .default_val(SERVICE_CLASSES[0])
            .display(service_class_display)),
        Box::new(TextInputNode::new("subdomain", "form.service.subdomain", 0, false)
            .hint("form.service.subdomain.hint")),
        Box::new(TextInputNode::new("alias",     "form.service.alias",     0, false)
            .hint("form.service.alias.hint")),
        // ── Tab 1: Options ─────────────────────────────────────────────────
        Box::new(TextInputNode::new("version", "form.options.version", 1, false)
            .default_val("latest")),
        Box::new(TextInputNode::new("port",    "form.service.port",    1, false)),
    ];
    ResourceForm::new(ResourceKind::Service, SERVICE_TABS, nodes, None, service_on_change)
}
